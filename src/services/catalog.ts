import { invoke } from "@tauri-apps/api/core";
import { ImageRecord, FolderRecord } from "../types";

export async function openCatalog(path: string): Promise<void> {
  return invoke("open_catalog", { path });
}

export async function createCatalog(path: string): Promise<void> {
  return invoke("create_catalog", { path });
}

export async function importDirectory(dirPath: string): Promise<number> {
  return invoke<"import_directory", { dirPath: string }, number>(
    "import_directory",
    { dirPath }
  );
}

export async function listImages(
  offset: number,
  limit: number,
  ratingFilter?: number | null
): Promise<ImageRecord[]> {
  return invoke<"list_images", { offset: number; limit: number; ratingFilter?: number | null }, ImageRecord[]>(
    "list_images",
    { offset, limit, ratingFilter: ratingFilter ?? null }
  );
}

export async function countImages(): Promise<number> {
  return invoke("count_images");
}

export async function listFolders(): Promise<FolderRecord[]> {
  return invoke("list_folders");
}

export async function getThumbnailPath(imageId: number): Promise<string> {
  return invoke<"get_thumbnail_path", { imageId: number }, string>(
    "get_thumbnail_path",
    { imageId }
  );
}

export async function getPreviewPath(imageId: number): Promise<string> {
  return invoke<"get_preview_path", { imageId: number }, string>(
    "get_preview_path",
    { imageId }
  );
}

export async function startFaceIndex(): Promise<void> {
  return invoke("start_face_index");
}

export async function upscaleImage(
  imagePath: string,
  outputDir: string,
  scale: number
): Promise<string> {
  return invoke<"upscale_image", { imagePath: string; outputDir: string; scale: number }, string>(
    "upscale_image",
    { imagePath, outputDir, scale }
  );
}
