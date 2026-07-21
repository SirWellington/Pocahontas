use std::fs::File;
use std::io::Read;
use anyhow::Result;

/// Parsed EXIF metadata from an image file.
#[derive(Debug, Clone, Default)]
pub struct ExifData {
    // Dimensions
    pub width: u32,
    pub height: u32,

    // Camera
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub software: Option<String>,

    // Exposure
    pub iso: Option<u32>,
    pub aperture: Option<f64>,
    pub shutter_num: Option<u32>,
    pub shutter_den: Option<u32>,
    pub focal_length: Option<f64>,
    pub flash_fired: Option<bool>,
    pub exposure_mode: Option<String>,
    pub white_balance: Option<String>,

    // Date/time
    pub date_time_original: Option<String>,

    // GPS
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
    pub gps_altitude: Option<f64>,
    pub gps_date_time: Option<String>,

    // Image info
    pub orientation: Option<u32>,
    pub bits_per_sample: Option<u32>,
    pub color_space: Option<String>,
}

pub struct ExifParser;

impl ExifParser {
    pub fn new() -> Self {
        Self
    }

    /// Parses EXIF data from an image file.
    /// Uses the `image` crate's EXIF reader for broad format support.
    /// For RAW formats (ARW, CR3, NEF, etc.), falls back to basic header parsing.
    pub fn parse(&self, path: &str) -> Result<ExifData> {
        let mut data = ExifData::default();

        // Try to open with the image crate first
        if let Ok(img) = image::io::Reader::open(path) {
            if let Ok(img) = img.with_guessed_format() {
                let metadata = img.header()?.exif();

                if let Some(exif) = metadata {
                    data = self.parse_exif_bytes(exif)?;
                }

                // Get dimensions from header
                let hdr = img.header()?;
                data.width = hdr.width();
                data.height = hdr.height();
            }
        }

        // For RAW formats not fully supported by the image crate,
        // try exiftool as fallback (if available on the system)
        if data.camera_model.is_none() {
            if let Ok(tool_data) = self.parse_with_exiftool(path) {
                data = tool_data;
            }
        }

        Ok(data)
    }

    /// Parses raw EXIF bytes into structured data.
    fn parse_exif_bytes(&self, exif_bytes: &[u8]) -> Result<ExifData> {
        let mut data = ExifData::default();

        if let Ok(ifh) = exif::Reader::new().read_from_buffer(exif_bytes) {
            for entry in ifh.iter() {
                match entry.ifd {
                    exif::IFD::Primary => {
                        match entry.entry_type {
                            exif::EntryType::Make => {
                                data.camera_make = entry.value.display_string().map(|s| s.to_string());
                            }
                            exif::EntryType::Model => {
                                data.camera_model = entry.value.display_string().map(|s| s.to_string());
                            }
                            exif::EntryType::DateTimeOriginal => {
                                data.date_time_original = entry.value.display_string().map(|s| s.to_string());
                            }
                            exif::EntryType::Software => {
                                data.software = entry.value.display_string().map(|s| s.to_string());
                            }
                            exif::EntryType::ExifOffset => {
                                // Points to Exif IFD
                            }
                            _ => {}
                        }
                    }
                    exif::IFD::Exif => {
                        match entry.entry_type {
                            exif::EntryType::ISOSpeedRatings => {
                                data.iso = entry.value.get_u32().ok();
                            }
                            exif::EntryType::FNumber => {
                                data.aperture = entry.value.get_rational().ok().map(|r| r.to_f64());
                            }
                            exif::EntryType::ExposureTime => {
                                let rational = entry.value.get_rational().ok();
                                if let Some(r) = rational {
                                    data.shutter_num = Some(r.numerator as u32);
                                    data.shutter_den = Some(r.denominator as u32);
                                }
                            }
                            exif::EntryType::FocalLength => {
                                data.focal_length = entry.value.get_rational().ok().map(|r| r.to_f64());
                            }
                            exif::EntryType::Flash => {
                                let val = entry.value.get_u32().ok();
                                data.flash_fired = val.map(|v| v & 1 != 0);
                            }
                            exif::EntryType::ExposureProgram => {
                                data.exposure_mode = entry.value.display_string().map(|s| s.to_string());
                            }
                            exif::EntryType::WhiteBalance => {
                                data.white_balance = entry.value.display_string().map(|s| s.to_string());
                            }
                            exif::EntryType::LensMake => {}
                            exif::EntryType::LensModel => {
                                data.lens_model = entry.value.display_string().map(|s| s.to_string());
                            }
                            exif::EntryType::ColorSpace => {
                                data.color_space = entry.value.display_string().map(|s| s.to_string());
                            }
                            _ => {}
                        }
                    }
                    exif::IFD::GPS => {
                        match entry.entry_type {
                            exif::EntryType::GPSLatitude => {
                                data.gps_latitude = self.parse_gps_coordinate(entry);
                            }
                            exif::EntryType::GPSLongitude => {
                                data.gps_longitude = self.parse_gps_coordinate(entry);
                            }
                            exif::EntryType::GPSLatitudeRef => {}
                            exif::EntryType::GPSLongitudeRef => {}
                            exif::EntryType::GPSAltitude => {
                                data.gps_altitude = entry.value.get_rational().ok().map(|r| r.to_f64());
                            }
                            exif::EntryType::GPSDateStamp => {
                                data.gps_date_time = entry.value.display_string().map(|s| s.to_string());
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(data)
    }

    fn parse_gps_coordinate(&self, entry: &exif::ExifValue) -> Option<f64> {
        entry.get_rational().ok().and_then(|r| {
            // EXIF GPS is stored as [degrees, minutes, seconds]
            // We just return a rough approximation
            Some(r.to_f64())
        })
    }

    /// Fallback: uses exiftool CLI if available on the system.
    fn parse_with_exiftool(&self, path: &str) -> Result<ExifData> {
        let output = std::process::Command::new("exiftool")
            .args(["-json", path])
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("exiftool failed"));
        }

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let obj = json.as_array().and_then(|a| a.first()).and_then(|v| v.as_object());

        let mut data = ExifData::default();

        if let Some(obj) = obj {
            data.camera_make = obj.get("Make").and_then(|v| v.as_str()).map(|s| s.to_string());
            data.camera_model = obj.get("Model").and_then(|v| v.as_str()).map(|s| s.to_string());
            data.lens_model = obj.get("LensModel").and_then(|v| v.as_str()).map(|s| s.to_string());
            data.iso = obj.get("ISO").and_then(|v| v.as_str()).and_then(|s| s.parse().ok());
            data.date_time_original = obj.get("DateTimeOriginal").and_then(|v| v.as_str()).map(|s| s.to_string());

            if let Some(ap) = obj.get("FNumber").and_then(|v| v.as_str()) {
                data.aperture = ap.trim_start_matches('f').parse().ok();
            }

            if let Some(shutter) = obj.get("ExposureTime").and_then(|v| v.as_str()) {
                let parts: Vec<&str> = shutter.split('/').collect();
                if parts.len() == 2 {
                    data.shutter_num = parts[0].parse().ok();
                    data.shutter_den = parts[1].parse().ok();
                }
            }

            if let Some(fl) = obj.get("FocalLength").and_then(|v| v.as_str()) {
                data.focal_length = fl.trim_end_matches(" mm").parse().ok();
            }

            // GPS
            data.gps_latitude = obj.get("GPSLatitude").and_then(|v| v.as_str()).and_then(|s| {
                parse_dms(s)
            });
            data.gps_longitude = obj.get("GPSLongitude").and_then(|v| v.as_str()).and_then(|s| {
                parse_dms(s)
            });
        }

        Ok(data)
    }
}

/// Parses DMS (degrees, minutes, seconds) string from exiftool into decimal degrees.
fn parse_dms(dms: &str) -> Option<f64> {
    let parts: Vec<f64> = dms
        .trim()
        .trim_end_matches('°')
        .split_whitespace()
        .filter_map(|p| p.trim_end_matches([' ', '\'', '"']).parse().ok())
        .collect();

    if parts.len() >= 3 {
        Some(parts[0] + parts[1] / 60.0 + parts[2] / 3600.0)
    } else if parts.len() == 1 {
        Some(parts[0])
    } else {
        None
    }
}
