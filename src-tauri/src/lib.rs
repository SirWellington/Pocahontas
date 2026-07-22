mod db;
mod catalog;
mod image;
mod exif;
#[cfg(feature = "face-detection")]
mod face;
mod python;

use anyhow::Result;
use tokio::sync::Mutex;
use tauri::{Manager, State};
use catalog::manager::CatalogManager;

/// App-level state shared across Tauri commands.
pub struct AppState {
    pub catalog: Mutex<Option<CatalogManager>>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Opens an existing .praetorian catalog file.
#[tauri::command]
async fn open_catalog(path: String, state: State<'_, AppState>) -> Result<String, String> {
    let catalog = CatalogManager::new(&path)
        .await
        .map_err(|e| e.to_string())?;

    let mut guard = state.catalog.lock().await;
    *guard = Some(catalog);
    Ok(path)
}

/// Creates a new .praetorian catalog at the given path.
#[tauri::command]
async fn create_catalog(path: String, state: State<'_, AppState>) -> Result<String, String> {
    let catalog = CatalogManager::new(&path)
        .await
        .map_err(|e| e.to_string())?;

    let mut guard = state.catalog.lock().await;
    *guard = Some(catalog);
    Ok(path)
}

/// Scans a directory and imports all supported images into the catalog.
#[tauri::command]
async fn import_directory(
    dir_path: String,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let pool = {
        let guard = state.catalog.lock().await;
        let catalog = guard.as_ref().ok_or_else(|| "No catalog open".to_string())?;
        catalog.db.pool().clone()
    };

    let imported = CatalogManager::import_directory_from_pool(&pool, &dir_path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(imported)
}

/// Lists images from the catalog with pagination and optional rating filter.
#[tauri::command]
async fn list_images(
    offset: i64,
    limit: i64,
    rating_filter: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vec<catalog::manager::ImageRecord>, String> {
    let pool = {
        let guard = state.catalog.lock().await;
        let catalog = guard.as_ref().ok_or_else(|| "No catalog open".to_string())?;
        catalog.db.pool().clone()
    };

    let images = CatalogManager::list_images_from_pool(&pool, offset, limit, rating_filter)
        .await
        .map_err(|e| e.to_string())?;

    Ok(images)
}

/// Returns the total number of images in the catalog.
#[tauri::command]
async fn count_images(state: State<'_, AppState>) -> Result<i64, String> {
    let pool = {
        let guard = state.catalog.lock().await;
        let catalog = guard.as_ref().ok_or_else(|| "No catalog open".to_string())?;
        catalog.db.pool().clone()
    };

    let count = CatalogManager::count_images_from_pool(&pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(count)
}

/// Lists all tracked folders in the catalog.
#[tauri::command]
async fn list_folders(
    state: State<'_, AppState>,
) -> Result<Vec<catalog::manager::FolderRecord>, String> {
    let pool = {
        let guard = state.catalog.lock().await;
        let catalog = guard
            .as_ref()
            .ok_or_else(|| "No catalog open".to_string())?;
        catalog.db.pool().clone()
    };

    let folders = CatalogManager::list_folders_from_pool(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(folders)
}

/// Returns the filesystem path to a thumbnail for a given image.
#[tauri::command]
async fn get_thumbnail_path(
    image_id: i64,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let path = {
        let guard = state.catalog.lock().await;
        let catalog = guard
            .as_ref()
            .ok_or_else(|| "No catalog open".to_string())?;

        catalog.get_thumbnail_path(image_id).to_string_lossy().to_string()
    };
    Ok(path)
}

/// Returns the filesystem path to a smart preview for a given image.
#[tauri::command]
async fn get_preview_path(
    image_id: i64,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let path = {
        let guard = state.catalog.lock().await;
        let catalog = guard
            .as_ref()
            .ok_or_else(|| "No catalog open".to_string())?;

        catalog.get_preview_path(image_id).to_string_lossy().to_string()
    };
    Ok(path)
}

/// Starts the background face detection index for all unindexed images.
#[tauri::command]
async fn start_face_index(state: State<'_, AppState>) -> Result<(), String> {
    let pool = {
        let guard = state.catalog.lock().await;
        let catalog = guard
            .as_ref()
            .ok_or_else(|| "No catalog open".to_string())?;
        catalog.db.pool().clone()
    };

    #[cfg(feature = "face-detection")]
    {
        let mut detector = face::detector::FaceDetector::new("retinaface.onnx");
        if let Err(e) = detector.initialize() {
            log::warn!("FaceDetector initialization failed, falling back to CPU: {}", e);
        }
        face::detector::rebuild_face_index(&pool, &detector).await.map_err(|e| e.to_string())?;
    }

    #[cfg(not(feature = "face-detection"))]
    {
        let _ = pool;
    }

    Ok(())
}

/// Upscales an image using the Python bridge.
#[tauri::command]
async fn upscale_image(
    image_path: String,
    output_dir: String,
    scale: u32,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    let bridge = python::bridge::PythonBridge::with_defaults();

    let result = bridge
        .upscale_image(&image_path, &output_dir, scale)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result.output_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppState {
                catalog: Mutex::new(None),
            });

            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::from_default_env()
                        .add_directive("praetorian=debug".parse().unwrap()),
                )
                .init();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            open_catalog,
            create_catalog,
            import_directory,
            list_images,
            count_images,
            list_folders,
            get_thumbnail_path,
            get_preview_path,
            start_face_index,
            upscale_image,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
