use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use anyhow::Result;

use sqlx::SqlitePool;

/// Thumbnail dimensions for the grid view.
const THUMB_MAX_DIM: u32 = 160;

/// Smart preview maximum long edge.
const PREVIEW_MAX_DIM: u32 = 2048;

/// JPEG quality for thumbnails (0-100).
const THUMB_QUALITY: u8 = 75;

/// JPEG quality for smart previews.
const PREVIEW_QUALITY: u8 = 85;

/// Maximum concurrent image processing tasks.
const MAX_CONCURRENT_TASKS: usize = 4;

/// Represents a queued image processing job.
#[derive(Debug, Clone)]
pub struct ProcessingJob {
    pub image_id: i64,
    pub source_path: String,
    pub job_type: JobType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JobType {
    Thumbnail,
    Preview,
}

/// The image processing pipeline. Runs as a background worker pool
/// driven by a tokio task and a channel-based job queue.
pub struct ImagePipeline {
    cache_dir: PathBuf,
    sender: mpsc::Sender<ProcessingJob>,
    receiver: Option<mpsc::Receiver<ProcessingJob>>,
    worker_handle: Option<JoinHandle<()>>,
}

impl ImagePipeline {
    pub fn new(cache_dir: PathBuf) -> Self {
        let (sender, receiver) = mpsc::channel(512);
        Self {
            cache_dir,
            sender,
            receiver: Some(receiver),
            worker_handle: None,
        }
    }

    /// Starts the background processing worker.
    /// Spawns a tokio task that drains the job queue and processes images
    /// using libvips without blocking the Tauri UI thread.
    pub fn start(&mut self, pool: Arc<SqlitePool>) -> &mut Self {
        let receiver = self
            .receiver
            .take()
            .expect("Worker already started; receiver was consumed");
        let cache_dir = self.cache_dir.clone();

        let handle = tokio::spawn(async move {
            Self::worker_loop(receiver, cache_dir, pool).await;
        });

        self.worker_handle = Some(handle);
        self
    }

    /// Submits a job to the processing queue.
    pub async fn submit(&self, job: ProcessingJob) -> Result<()> {
        self.sender
            .send(job)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Submits thumbnail generation for a batch of images.
    pub async fn queue_thumbnails(&self, images: &[(i64, String)]) -> Result<()> {
        for (image_id, source_path) in images {
            self.submit(ProcessingJob {
                image_id: *image_id,
                source_path: source_path.clone(),
                job_type: JobType::Thumbnail,
            })
            .await?;
        }
        Ok(())
    }

    /// Submits smart preview generation for a batch of images.
    pub async fn queue_previews(&self, images: &[(i64, String)]) -> Result<()> {
        for (image_id, source_path) in images {
            self.submit(ProcessingJob {
                image_id: *image_id,
                source_path: source_path.clone(),
                job_type: JobType::Preview,
            })
            .await?;
        }
        Ok(())
    }

    /// Core worker loop: processes jobs from the channel using libvips.
    /// Uses a semaphore-like pattern to limit concurrency.
    async fn worker_loop(
        mut receiver: mpsc::Receiver<ProcessingJob>,
        cache_dir: PathBuf,
        pool: Arc<SqlitePool>,
    ) {
        let mut active_tasks: Vec<JoinHandle<()>> = Vec::new();

        while let Some(job) = receiver.recv().await {
            // Limit concurrent tasks
            while active_tasks.len() >= MAX_CONCURRENT_TASKS {
                active_tasks.retain(|t| !t.is_finished());
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }

            let source = job.source_path.clone();
            let image_id = job.image_id;
            let job_type = job.job_type.clone();
            let out_dir = cache_dir.clone();
            let db_pool = pool.clone();

            let task = tokio::spawn(async move {
                match job_type {
                    JobType::Thumbnail => {
                        if let Err(e) =
                            Self::generate_thumbnail(&source, image_id, &out_dir, &db_pool).await
                        {
                            log::error!(
                                "Failed to generate thumbnail for image {}: {}",
                                image_id,
                                e
                            );
                        }
                    }
                    JobType::Preview => {
                        if let Err(e) =
                            Self::generate_preview(&source, image_id, &out_dir, &db_pool).await
                        {
                            log::error!(
                                "Failed to generate preview for image {}: {}",
                                image_id,
                                e
                            );
                        }
                    }
                }
            });

            active_tasks.push(task);
        }
    }

    /// Generates a tiny thumbnail using libvips.
    async fn generate_thumbnail(
        source_path: &str,
        image_id: i64,
        cache_dir: &Path,
        pool: &SqlitePool,
    ) -> Result<()> {
        let output_path = cache_dir.join(format!("thumb_{}.jpg", image_id));

        let result = Self::resize_and_save(
            source_path,
            &output_path,
            THUMB_MAX_DIM,
            THUMB_QUALITY,
        );

        match result {
            Ok(_) => {
                sqlx::query!(
                    "UPDATE images SET has_thumbnail = 1, thumbnail_hash = ? WHERE id = ?",
                    output_path.file_name().unwrap().to_string_lossy(),
                    image_id,
                )
                .execute(pool)
                .await?;
                log::debug!("Generated thumbnail for image {}", image_id);
            }
            Err(e) => {
                log::error!("vips thumbnail error for {}: {}", source_path, e);
                return Err(e);
            }
        }

        Ok(())
    }

    /// Generates a smart preview (2048px max) using libvips.
    async fn generate_preview(
        source_path: &str,
        image_id: i64,
        cache_dir: &Path,
        pool: &SqlitePool,
    ) -> Result<()> {
        let output_path = cache_dir.join(format!("preview_{}.jpg", image_id));

        let result = Self::resize_and_save(
            source_path,
            &output_path,
            PREVIEW_MAX_DIM,
            PREVIEW_QUALITY,
        );

        match result {
            Ok(_) => {
                sqlx::query!(
                    "UPDATE images SET has_preview = 1, preview_hash = ? WHERE id = ?",
                    output_path.file_name().unwrap().to_string_lossy(),
                    image_id,
                )
                .execute(pool)
                .await?;
                log::debug!("Generated preview for image {}", image_id);
            }
            Err(e) => {
                log::error!("vips preview error for {}: {}", source_path, e);
                return Err(e);
            }
        }

        Ok(())
    }

    /// Uses libvips to resize an image to the given max dimension and save as JPEG.
    /// This is the core vips operation that handles RAW, TIFF, and standard formats.
    fn resize_and_save(
        input: &str,
        output: &Path,
        max_dim: u32,
        quality: u8,
    ) -> Result<()> {
        let img = vips::image::load(input)?;

        let (width, height) = (img.width(), img.height());
        let scale = if width > height {
            max_dim as f64 / width as f64
        } else {
            max_dim as f64 / height as f64
        };

        if scale >= 1.0 {
            // Image is already smaller than target; just convert to JPEG
            img.write_jpeg(
                output.to_str().unwrap(),
                vips::image::JpegOptions {
                    Q: quality as i32,
                    ..Default::default()
                },
            )?;
        } else {
            let resized = img.resize(vips::image::ResizeOptions {
                scale_factor: Some(scale),
                ..Default::default()
            })?;

            resized.write_jpeg(
                output.to_str().unwrap(),
                vips::image::JpegOptions {
                    Q: quality as i32,
                    ..Default::default()
                },
            )?;
        }

        Ok(())
    }
}
