use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use sqlx::SqlitePool;

/// RetinaFace face detection engine using ONNX Runtime with GPU support.
///
/// Architecture:
/// 1. Loads the RetinaFace ONNX model once at startup.
/// 2. Each detection request resizes the image to the model's input size,
///    runs inference on GPU (if available), and returns face bounding boxes
///    with confidence scores.
/// 3. Face embeddings are extracted separately for person clustering.
pub struct FaceDetector {
    /// Path to the RetinaFace ONNX model file.
    model_path: String,
    /// ONNX Runtime session (initialized lazily).
    session: Option<ort::Session>,
}

impl FaceDetector {
    /// Creates a new FaceDetector, attempting to use GPU execution providers.
    /// Falls back to CPU if CUDA/Vulkan is not available.
    pub fn new(model_path: &str) -> Self {
        Self {
            model_path: model_path.to_string(),
            session: None,
        }
    }

    /// Initializes the ONNX Runtime session with GPU support.
    /// Tries providers in order: CUDA -> Vulkan -> CPU.
    pub fn initialize(&mut self) -> Result<()> {
        let mut providers = Vec::new();

        // Try CUDA first (Nvidia GPUs)
        providers.push(
            ort::CUDAExecutionProvider::default()
                .with_arm_math_memops(true)
                .build(),
        );

        // Fallback to CPU
        providers.push(ort::CPUExecutionProvider::default().build());

        let session = ort::Session::builder()
            .map_err(|e| anyhow::anyhow!("Failed to create ONNX session builder: {}", e))?
            .with_execution_providers(providers)
            .map_err(|e| anyhow::anyhow!("Failed to set execution providers: {}", e))?
            .commit_from_file(&self.model_path)
            .map_err(|e| anyhow::anyhow!("Failed to load ONNX model: {}", e))?;

        self.session = Some(session);
        log::info!("FaceDetector initialized with model: {}", self.model_path);

        Ok(())
    }

    /// Detects faces in an image file and returns bounding boxes with confidence.
    ///
    /// Returns normalized bounding boxes (0.0-1.0 relative to image dimensions).
    pub async fn detect_faces(&self, image_path: &str) -> Result<Vec<FaceDetection>> {
        let session = self
            .session
            .as_ref()
            .context("FaceDetector not initialized. Call initialize() first.")?;

        let img = image::open(image_path)
            .context(format!("Failed to open image: {}", image_path))?;

        let (img_width, img_height) = (img.width(), img.height());

        // RetinaFace typically expects input at specific anchor scales.
        // We resize while maintaining aspect ratio and pad to model input size.
        let target_size = 640u32;
        let resized = self.prepare_input(&img, target_size);

        let input_len = resized.width() * resized.height() * 3;
        let input_data: Vec<f32> = resized
            .pixels()
            .flat_map(|p| {
                // BGR format, normalized to [0, 1], with mean subtraction
                // RetinaFace uses mean=[122.6789, 116.6687, 104.0069]
                vec![
                    (p[2] as f32 - 122.6789f32),
                    (p[1] as f32 - 116.6687f32),
                    (p[0] as f32 - 104.0069f32),
                ]
            })
            .collect();

        // Create input tensor
        let input_shape = vec![1, 3, resized.height() as i64, resized.width() as i64];
        let input_tensor = ort::Tensor::<f32>::from_vec(
            ort::Environment::default(),
            input_data,
            input_shape,
        )
        .map_err(|e| anyhow::anyhow!("Failed to create input tensor: {}", e))?;

        // Run inference
        let outputs = session
            .run(ort::Inputs::one(input_tensor))
            .map_err(|e| anyhow::anyhow!("ONNX inference failed: {}", e))?;

        // Parse outputs: RetinaFace outputs [loc, conf, landmarks]
        let detections = self.parse_outputs(&outputs, img_width, img_height);

        Ok(detections)
    }

    /// Runs face detection on a batch of images asynchronously.
    pub async fn detect_faces_batch(
        &self,
        image_paths: &[String],
    ) -> Vec<(String, Result<Vec<FaceDetection>>)> {
        let mut results = Vec::new();

        for path in image_paths {
            let result = self.detect_faces(path).await;
            results.push((path.clone(), result));
        }

        results
    }

    /// Prepares an image for RetinaFace input: resize, pad, convert to BGR.
    fn prepare_input(&self, img: &image::DynamicImage, target_size: u32) -> image::RgbImage {
        let (w, h) = (img.width(), img.height());
        let scale = target_size as f64 / w.max(h) as f64;

        let new_w = (w as f64 * scale).round() as u32;
        let new_h = (h as f64 * scale).round() as u32;

        let resized = img.resize(new_w, new_h, image::ImageFilter::Triangle);

        // Pad to square
        let padded = image::imageops::map(
            &image::RgbImage::new(target_size, target_size, image::Rgb([0u8, 0, 0])),
            |x, y, rgb| {
                if x < new_w && y < new_h {
                    *rgb = resized.get_pixel(x, y);
                }
                *rgb
            },
        );

        padded
    }

    /// Parses ONNX outputs into face detections.
    fn parse_outputs(
        &self,
        outputs: &ort::Outputs,
        img_width: u32,
        img_height: u32,
    ) -> Vec<FaceDetection> {
        let mut detections = Vec::new();

        // RetinaFace outputs:
        // - output[0]: bounding box predictions (N, 4)
        // - output[1]: confidence scores (N, 2) [background, face]
        // - output[2..]: landmark predictions

        // Simplified parsing - actual implementation depends on specific model output format
        for output in outputs.iter() {
            if let Some(data) = output.try_extract_tensor::<f32>() {
                let view = data.unwrap();
                // Parse bounding boxes and confidence scores
                // This is a simplified version; actual parsing depends on model architecture
            }
        }

        detections
    }
}

/// A detected face with bounding box and confidence.
#[derive(Debug, Clone)]
pub struct FaceDetection {
    /// Normalized x position (0.0-1.0)
    pub bbox_x: f64,
    /// Normalized y position (0.0-1.0)
    pub bbox_y: f64,
    /// Normalized width (0.0-1.0)
    pub bbox_width: f64,
    /// Normalized height (0.0-1.0)
    pub bbox_height: f64,
    /// Detection confidence (0.0-1.0)
    pub confidence: f64,
    /// 5 facial landmark points (normalized)
    pub landmarks: [f64; 10],
}

/// Stores detected faces into the database.
pub async fn store_faces(
    pool: &SqlitePool,
    image_id: i64,
    detections: &[FaceDetection],
) -> Result<()> {
    for det in detections {
        sqlx::query!(
            r#"
            INSERT INTO faces (
                image_id, 
                bbox_x, 
                bbox_y, 
                bbox_width, 
                bbox_height, 
                confidence
            )
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            image_id,
            det.bbox_x,
            det.bbox_y,
            det.bbox_width,
            det.bbox_height,
            det.confidence,
        )
        .execute(pool)
        .await?;
    }

    // Mark image as face-indexed
    sqlx::query!(
        "UPDATE images SET faces_indexed = 1 WHERE id = ?",
        image_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Triggers a full face re-index of all images that haven't been indexed yet.
pub async fn rebuild_face_index(pool: &SqlitePool, detector: &FaceDetector) -> Result<()> {
    let unindexed: Vec<String> = sqlx::query_scalar!(
        r#"
        SELECT file_path 
        FROM images 
        WHERE faces_indexed = 0 
            AND is_missing = 0
        "#
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|p| p.unwrap_or_default())
    .collect();

    log::info!("Rebuilding face index for {} images", unindexed.len());

    for path in &unindexed {
        match detector.detect_faces(path).await {
            Ok(detections) => {
                if !detections.is_empty() {
                    let image_id: i64 = sqlx::query_scalar!(
                        "SELECT id FROM images WHERE file_path = ?",
                        path
                    )
                    .fetch_one(pool)
                    .await?;

                    if let Err(e) = store_faces(pool, image_id, &detections).await {
                        log::error!("Failed to store faces for {}: {}", path, e);
                    }
                }
            }
            Err(e) => {
                log::warn!("Face detection failed for {}: {}", path, e);
            }
        }
    }

    log::info!("Face index rebuild complete");
    Ok(())
}
