import { invoke } from "@tauri-apps/api/core";
import { ImageRecord, FolderRecord } from "../types";

export async function openCatalog(path: string): Promise<void> {
  return invoke<void>("open_catalog", { path });
}

export async function createCatalog(path: string): Promise<void> {
  return invoke<void>("create_catalog", { path });
}

export async function importDirectory(dirPath: string): Promise<number> {
  return invoke<number>("import_directory", { dirPath });
}

export async function listImages(
  offset: number,
  limit: number,
  ratingFilter?: number | null
): Promise<ImageRecord[]> {
  return invoke<ImageRecord[]>("list_images", {
    offset,
    limit,
    ratingFilter: ratingFilter ?? null,
  });
}

export async function countImages(): Promise<number> {
  return invoke<number>("count_images");
}

export async function listFolders(): Promise<FolderRecord[]> {
  return invoke<FolderRecord[]>("list_folders");
}

export async function getThumbnailPath(imageId: number): Promise<string> {
  return invoke<string>("get_thumbnail_path", { imageId });
}

export async function getPreviewPath(imageId: number): Promise<string> {
  return invoke<string>("get_preview_path", { imageId });
}

export async function startFaceIndex(): Promise<void> {
  return invoke<void>("start_face_index");
}

export async function upscaleImage(
  imagePath: string,
  outputDir: string,
  scale: number
): Promise<string> {
  return invoke<string>("upscale_image", { imagePath, outputDir, scale });
}
