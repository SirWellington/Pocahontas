use std::path::{Path, PathBuf};
use std::fs;

use sqlx::{SqlitePool, FromRow};
use sha2::{Sha256, Digest};
use walkdir::WalkDir;
use anyhow::{Context, Result};

use crate::db::Database;
use crate::exif::parser::ExifParser;
use crate::image::pipeline::ImagePipeline;

const SUPPORTED_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "tiff", "tif", "webp", "bmp",
    "arw", "cr3", "cr2", "nef", "orf", "sr2", "dng", "raf",
];

#[derive(Debug, FromRow)]
pub struct ImageRecord {
    pub id: i64,
    pub folder_id: Option<i64>,
    pub file_path: String,
    pub file_name: String,
    pub file_extension: String,
    pub file_size_bytes: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub date_taken: Option<String>,
    pub date_imported: String,
    pub has_thumbnail: bool,
    pub has_preview: bool,
    pub rating: i32,
    pub is_favorite: bool,
    pub faces_indexed: bool,
    pub is_missing: bool,
}

#[derive(Debug, FromRow)]
pub struct FolderRecord {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub parent_id: Option<i64>,
    pub date_added: String,
    pub is_watched: bool,
}

pub struct CatalogManager {
    pub db: Database,
    pub cache_dir: PathBuf,
    pub exif_parser: ExifParser,
    pub image_pipeline: ImagePipeline,
}

impl CatalogManager {
    pub async fn new(catalog_path: &str) -> Result<Self> {
        let db = Database::open_or_create(catalog_path).await
            .context("Failed to open or create catalog database")?;

        let catalog_dir = Path::new(catalog_path)
            .parent()
            .context("Catalog path has no parent directory")?
            .to_path_buf();

        let cache_dir = catalog_dir.join("cache");
        fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;

        let exif_parser = ExifParser::new();
        let image_pipeline = ImagePipeline::new(cache_dir.clone());

        Ok(Self {
            db,
            cache_dir,
            exif_parser,
            image_pipeline,
        })
    }

    /// Scans a directory and imports all supported image files into the catalog.
    /// Returns the count of newly imported images.
    pub async fn import_directory(&self, dir_path: &str) -> Result<usize> {
        let pool = self.db.pool();
        let dir = Path::new(dir_path);

        if !dir.exists() {
            anyhow::bail!("Directory does not exist: {}", dir_path);
        }

        // Insert or get folder record
        let folder_id = self.ensure_folder(dir_path, pool).await?;

        let mut imported = 0usize;
        let mut queue = vec![dir.to_path_buf()];

        while let Some(current_dir) = queue.pop() {
            for entry in WalkDir::new(&current_dir).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_dir() {
                    if entry.file_name() != ".praetorian" {
                        queue.push(entry.path().to_path_buf());
                    }
                    continue;
                }

                let ext = entry
                    .path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_lowercase())
                    .unwrap_or_default();

                if !SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
                    continue;
                }

                let file_path = entry.path().to_string_lossy().to_string();

                // Skip if already cataloged
                let exists: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM images WHERE file_path = ?)",
                )
                .bind(&file_path)
                .fetch_one(pool)
                .await?;

                if exists {
                    continue;
                }

                match self.import_single_image(&file_path, folder_id, pool).await {
                    Ok(_) => imported += 1,
                    Err(e) => {
                        log::warn!("Failed to import {}: {}", file_path, e);
                    }
                }
            }
        }

        Ok(imported)
    }

    /// Imports a single image file: reads EXIF, creates DB record, queues thumbnail generation.
    async fn import_single_image(
        &self,
        file_path: &str,
        folder_id: i64,
        pool: &SqlitePool,
    ) -> Result<i64> {
        let metadata = fs::metadata(file_path)
            .context(format!("Cannot access file: {}", file_path))?;

        let file_name = Path::new(file_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let file_extension = Path::new(file_path)
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
            .to_lowercase();

        let file_hash = self.compute_file_hash(file_path)?;

        // Parse EXIF metadata
        let exif = self.exif_parser
            .parse(file_path)
            .unwrap_or_default();

        // Insert image record
        let result = sqlx::query!(
            r#"
            INSERT INTO images (
                folder_id, 
                file_path, 
                file_name, 
                file_extension,
                file_size_bytes,
                width, 
                height, 
                date_taken, 
                file_hash
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            folder_id,
            file_path,
            file_name,
            file_extension,
            metadata.len() as i64,
            exif.width as Option<i32>,
            exif.height as Option<i32>,
            exif.date_time_original.as_deref(),
            file_hash,
        )
        .execute(pool)
        .await?;

        let image_id = result.last_insert_rowid();

        // Insert EXIF data
        sqlx::query!(
            r#"
            INSERT INTO exif_data (
                image_id, 
                camera_make, 
                camera_model, 
                lens_model,
                iso, 
                aperture_f_number,
                shutter_speed_num, 
                shutter_speed_den,
                focal_length_mm, 
                gps_latitude, 
                gps_longitude, 
                gps_altitude
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            image_id,
            exif.camera_make.as_deref(),
            exif.camera_model.as_deref(),
            exif.lens_model.as_deref(),
            exif.iso as Option<i32>,
            exif.aperture,
            exif.shutter_num as Option<i32>,
            exif.shutter_den as Option<i32>,
            exif.focal_length,
            exif.gps_latitude,
            exif.gps_longitude,
            exif.gps_altitude,
        )
        .execute(pool)
        .await?;

        // Queue thumbnail generation
        self.queue_thumbnail(image_id, file_path, pool).await?;

        Ok(image_id)
    }

    async fn ensure_folder(&self, path: &str, pool: &SqlitePool) -> Result<i64> {
        let folder_name = Path::new(path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let existing: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM folders WHERE path = ?",
        )
        .bind(path)
        .fetch_optional(pool)
        .await?;

        if let Some(id) = existing {
            return Ok(id);
        }

        let result = sqlx::query!(
            "INSERT INTO folders (path, name) VALUES (?, ?)",
            path,
            folder_name,
        )
        .execute(pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    async fn queue_thumbnail(
        &self,
        image_id: i64,
        file_path: &str,
        pool: &SqlitePool,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO processing_queue (
              image_id, 
              task_type, 
              priority
            )
            VALUES (?, 'thumbnail', 10)
            "#,
            image_id,
            file_path,
        )
        .execute(pool)
        .await?;

        // Also queue smart preview
        sqlx::query!(
            r#"
            INSERT INTO processing_queue (
                image_id, 
                task_type, 
                priority
            )
            VALUES (?, 'preview', 5)
            "#,
            image_id,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    fn compute_file_hash(path: &str) -> Result<String> {
        let contents = fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&contents);
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Fetches paginated images with optional filters.
    pub async fn list_images(
        &self,
        offset: i64,
        limit: i64,
        rating_filter: Option<i32>,
    ) -> Result<Vec<ImageRecord>> {
        let pool = self.db.pool();

        let images = sqlx::query_as::<_, ImageRecord>(
            r#"
            SELECT id, 
                   folder_id, 
                   file_path, 
                   file_name, 
                   file_extension,
                   file_size_bytes, 
                   width, 
                   height, 
                   date_taken, 
                   date_imported,
                   has_thumbnail, 
                   has_preview, 
                   rating, 
                   is_favorite,
                   faces_indexed, 
                   is_missing
            FROM images
            WHERE is_archived = 0
              AND ($1 IS NULL OR rating = $1)
            ORDER BY date_taken DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(rating_filter)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(images)
    }

    /// Gets the total image count (for pagination).
    pub async fn count_images(&self) -> Result<i64> {
        let pool = self.db.pool();
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM images WHERE is_archived = 0",
        )
        .fetch_one(pool)
        .await?;
        Ok(count)
    }

    /// Gets the cache directory path for a given image ID.
    pub fn get_thumbnail_path(&self, image_id: i64) -> PathBuf {
        self.cache_dir.join(format!("thumb_{}.jpg", image_id))
    }

    pub fn get_preview_path(&self, image_id: i64) -> PathBuf {
        self.cache_dir.join(format!("preview_{}.jpg", image_id))
    }
}
