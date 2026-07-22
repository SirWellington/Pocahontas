export interface ImageRecord {
  id: number;
  folder_id: number | null;
  file_path: string;
  file_name: string;
  file_extension: string;
  file_size_bytes: number;
  width: number | null;
  height: number | null;
  date_taken: string | null;
  date_imported: string;
  has_thumbnail: boolean;
  has_preview: boolean;
  rating: number;
  is_favorite: boolean;
  faces_indexed: boolean;
  is_missing: boolean;
  exif?: ExifData | null;
}

export interface ExifData {
  image_id: number;
  camera_make: string | null;
  camera_model: string | null;
  lens_model: string | null;
  iso: number | null;
  aperture_f_number: number | null;
  shutter_speed_num: number | null;
  shutter_speed_den: number | null;
  focal_length_mm: number | null;
  gps_latitude: number | null;
  gps_longitude: number | null;
  gps_altitude: number | null;
}

export interface FolderRecord {
  id: number;
  path: string;
  name: string;
  parent_id: number | null;
  date_added: string;
  is_watched: boolean;
}

export interface FaceRecord {
  id: number;
  image_id: number;
  person_id: number | null;
  bbox_x: number;
  bbox_y: number;
  bbox_width: number;
  bbox_height: number;
  confidence: number;
}

export interface PersonRecord {
  id: number;
  name: string;
  created_at: string;
  updated_at: string;
  face_count: number;
}

export interface TagRecord {
  id: number;
  name: string;
  color: string | null;
}

export interface AlbumRecord {
  id: number;
  name: string;
  description: string | null;
  created_at: string;
}

export interface CatalogStats {
  total_images: number;
  total_faces: number;
  total_people: number;
  total_folders: number;
  total_tags: number;
  unindexed_faces: number;
  missing_files: number;
}

export interface CatalogState {
  path: string | null;
  images: ImageRecord[];
  totalImages: number;
  isLoading: boolean;
  error: string | null;
}

export interface ProcessingJob {
  id: number;
  image_id: number;
  task_type: "thumbnail" | "preview" | "face_detect" | "upscale";
  status: "pending" | "running" | "completed" | "failed";
  priority: number;
  error_message: string | null;
  progress: number;
}

export type SidebarView = "library" | "develop" | "preview" | "map" | "people";
