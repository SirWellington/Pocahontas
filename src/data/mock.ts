import { ImageRecord, FolderRecord, ExifData } from "@/types";

const NAMES = [
  "DSC001", "DSC002", "DSC003", "DSC004", "DSC005",
  "IMG_4521", "IMG_4522", "IMG_4523", "IMG_4524", "IMG_4525",
  "P1040001", "P1040002", "P1040003", "P1040004", "P1040005",
  "R0987654", "R0987655", "R0987656", "R0987657", "R0987658",
  "A001", "A002", "A003", "A004", "A005",
  "MVI_0012", "MVI_0013", "MVI_0014", "MVI_0015", "MVI_0016",
  "D800_001", "D800_002", "D800_003", "D800_004", "D800_005",
  "X_T4521", "X_T4522", "X_T4523", "X_T4524", "X_T4525",
  "C506_0001", "C506_0002", "C506_0003", "C506_0004", "C506_0005",
  "ILFH8621", "ILFH8622", "ILFH8623", "ILFH8624", "ILFH8625",
  "MW320101", "MW320102", "MW320103", "MW320104", "MW320105",
];

const EXTENSIONS = ["DNG", "CR3", "ARW", "NEF", "RAF", "jpg", "CR2", "SR2"];
const CAMERAS = [
  { make: "Canon", model: "EOS R5" },
  { make: "Sony", model: "A7 IV" },
  { make: "Nikon", model: "Z 8" },
  { make: "Fujifilm", model: "X-T5" },
  { make: "Canon", model: "EOS 5D Mark IV" },
  { make: "Sony", model: "A7R V" },
  { make: "Nikon", model: "Z6 III" },
  { make: "Panasonic", model: "S5 II" },
];

const LENSES = [
  "RF 24-70mm f/2.8L IS USM",
  "FE 24-70mm f/2.8 GM II",
  "Z 24-70mm f/2.8 S",
  "XF 23mm f/1.4 R WM",
  "RF 70-200mm f/2.8L IS USM",
  "FE 85mm f/1.4 GM",
  "Z 85mm f/1.2 S",
  "RF 35mm f/1.8 Macro",
];

const LOCATIONS = [
  { lat: 37.7749, lon: -122.4194, label: "San Francisco, CA" },
  { lat: 40.7128, lon: -74.006, label: "New York, NY" },
  { lat: 51.5074, lon: -0.1278, label: "London, UK" },
  { lat: 48.8566, lon: 2.3522, label: "Paris, FR" },
  { lat: 35.6762, lon: 139.6503, label: "Tokyo, JP" },
  { lat: 34.0522, lon: -118.2437, label: "Los Angeles, CA" },
];

function seededRandom(seed: number) {
  let s = seed;
  return () => {
    s = (s * 16807 + 0) % 2147483647;
    return (s - 1) / 2147483646;
  };
}

export function generateMockImages(count: number = 60): ImageRecord[] {
  const rng = seededRandom(42);
  const images: ImageRecord[] = [];

  for (let i = 0; i < count; i++) {
    const name = NAMES[i % NAMES.length];
    const ext = EXTENSIONS[Math.floor(rng() * EXTENSIONS.length)].toLowerCase();
    const camera = CAMERAS[Math.floor(rng() * CAMERAS.length)];
    const lens = LENSES[Math.floor(rng() * LENSES.length)];
    const location = LOCATIONS[Math.floor(rng() * LOCATIONS.length)];
    const dayOffset = Math.floor(rng() * 90);
    const date = new Date(2025, 10, 15 - dayOffset);

    const width = [4256, 6000, 8192, 4928, 5472][Math.floor(rng() * 5)];
    const height = width > 5000 ? Math.round(width * 0.667) : Math.round(width * 0.75);

    images.push({
      id: i + 1,
      folder_id: Math.floor(rng() * 3) + 1,
      file_path: `/Volumes/SDCard/DCIM/100_CANON/${name}.${ext}`,
      file_name: `${name}.${ext}`,
      file_extension: ext,
      file_size_bytes: Math.floor(rng() * 80_000_000) + 15_000_000,
      width,
      height,
      date_taken: date.toISOString(),
      date_imported: new Date(2025, 11, 1).toISOString(),
      has_thumbnail: rng() > 0.15,
      has_preview: rng() > 0.3,
      rating: rng() > 0.7 ? Math.floor(rng() * 5) + 1 : 0,
      is_favorite: rng() > 0.85,
      faces_indexed: rng() > 0.4,
      is_missing: false,
      exif: {
        image_id: i + 1,
        camera_make: camera.make,
        camera_model: camera.model,
        lens_model: lens,
        iso: [100, 200, 400, 800, 1600, 3200, 6400][Math.floor(rng() * 7)],
        aperture: [1.4, 1.8, 2.0, 2.8, 4.0, 5.6, 8.0, 11, 16][Math.floor(rng() * 9)],
        shutter_speed_num: 1,
        shutter_speed_den: [60, 125, 250, 500, 1000, 2000, 4000, 8000][Math.floor(rng() * 8)],
        focal_length: [24, 35, 50, 70, 85, 105, 135, 200][Math.floor(rng() * 8)],
        gps_latitude: location.lat + (rng() - 0.5) * 0.01,
        gps_longitude: location.lon + (rng() - 0.5) * 0.01,
        gps_altitude: Math.floor(rng() * 500),
        date_time_original: date.toISOString(),
      },
    });
  }

  return images.sort((a, b) =>
    (b.date_taken ?? "").localeCompare(a.date_taken ?? "")
  );
}

export function generateMockFolders(): FolderRecord[] {
  return [
    { id: 1, path: "/Volumes/SDCard/DCIM/100_CANON", name: "100_CANON", parent_id: null, date_added: "2025-12-01T00:00:00Z", is_watched: true },
    { id: 2, path: "/Volumes/SDCard/DCIM/101_CANON", name: "101_CANON", parent_id: null, date_added: "2025-12-01T00:00:00Z", is_watched: true },
    { id: 3, path: "/Users/moreno/Photos/2025-Tokyo", name: "2025-Tokyo", parent_id: null, date_added: "2025-11-20T00:00:00Z", is_watched: false },
    { id: 4, path: "/Users/moreno/Photos/2025-Europe", name: "2025-Europe", parent_id: null, date_added: "2025-10-15T00:00:00Z", is_watched: false },
    { id: 5, path: "/Users/moreno/Photos/2025-Local", name: "2025-Local", parent_id: null, date_added: "2025-09-01T00:00:00Z", is_watched: false },
  ];
}

export function generateMockExif(imageId: number): ExifData | null {
  const images = generateMockImages(60);
  const img = images.find((i) => i.id === imageId);
  return img?.exif ?? null;
}
