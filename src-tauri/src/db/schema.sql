-- Praetorian Database Schema
-- Version: 1.0.0
-- Catalog file extension: .praetorian

-- ============================================
-- FOLDERS: Tracked import directories
-- ============================================
CREATE TABLE IF NOT EXISTS folders (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    path          TEXT NOT NULL UNIQUE,
    name          TEXT NOT NULL,
    parent_id     INTEGER REFERENCES folders(id) ON DELETE SET NULL,
    date_added    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    date_modified TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    is_watched    INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_folders_path ON folders(path);

-- ============================================
-- IMAGES: Core image records
-- ============================================
CREATE TABLE IF NOT EXISTS images (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    folder_id         INTEGER REFERENCES folders(id) ON DELETE SET NULL,
    file_path         TEXT NOT NULL UNIQUE,
    file_name         TEXT NOT NULL,
    file_extension    TEXT NOT NULL,
    file_size_bytes   INTEGER NOT NULL DEFAULT 0,
    width             INTEGER,
    height            INTEGER,
    date_taken        TEXT,
    date_modified     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    date_imported     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    -- Preview cache references (hashed filenames in cache/ folder)
    thumbnail_hash    TEXT,
    preview_hash      TEXT,

    -- Preview generation status
    has_thumbnail     INTEGER NOT NULL DEFAULT 0,
    has_preview       INTEGER NOT NULL DEFAULT 0,

    -- User metadata
    rating            INTEGER DEFAULT 0,
    is_favorite       INTEGER NOT NULL DEFAULT 0,
    is_archived       INTEGER NOT NULL DEFAULT 0,
    caption           TEXT,
    color_label       TEXT,

    -- Face index status
    faces_indexed     INTEGER NOT NULL DEFAULT 0,

    -- File integrity check
    file_hash         TEXT,
    is_missing        INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_images_folder_id ON images(folder_id);
CREATE INDEX IF NOT EXISTS idx_images_date_taken ON images(date_taken);
CREATE INDEX IF NOT EXISTS idx_images_rating ON images(rating);
CREATE INDEX IF NOT EXISTS idx_images_file_path ON images(file_path);
CREATE INDEX IF NOT EXISTS idx_images_faces_indexed ON images(faces_indexed);
CREATE INDEX IF NOT EXISTS idx_images_is_missing ON images(is_missing);

-- ============================================
-- EXIF_DATA: Detailed camera metadata per image
-- ============================================
CREATE TABLE IF NOT EXISTS exif_data (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    image_id          INTEGER NOT NULL UNIQUE REFERENCES images(id) ON DELETE CASCADE,

    -- Camera info
    camera_make       TEXT,
    camera_model      TEXT,
    lens_model        TEXT,
    software          TEXT,

    -- Exposure settings
    iso               INTEGER,
    aperture_f_number REAL,
    shutter_speed_num INTEGER,
    shutter_speed_den INTEGER,
    focal_length_mm   REAL,
    flash_fired       INTEGER,
    exposure_mode     TEXT,
    white_balance     TEXT,

    -- Image info
    orientation       INTEGER,
    bits_per_sample   INTEGER,
    color_space       TEXT,

    -- GPS
    gps_latitude      REAL,
    gps_longitude     REAL,
    gps_altitude      REAL,
    gps_date_time     TEXT
);

CREATE INDEX IF NOT EXISTS idx_exif_camera_model ON exif_data(camera_model);
CREATE INDEX IF NOT EXISTS idx_exif_lens_model ON exif_data(lens_model);
CREATE INDEX IF NOT EXISTS idx_exif_iso ON exif_data(iso);
CREATE INDEX IF NOT EXISTS idx_exif_gps_lat ON exif_data(gps_latitude);
CREATE INDEX IF NOT EXISTS idx_exif_gps_lon ON exif_data(gps_longitude);

-- ============================================
-- PEOPLE: Recognized persons (face clusters)
-- ============================================
CREATE TABLE IF NOT EXISTS people (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT NOT NULL,
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    -- Representative embedding (average of all face embeddings for this person)
    embedding     BLOB
);

CREATE INDEX IF NOT EXISTS idx_people_name ON people(name);

-- ============================================
-- FACES: Detected faces linked to images and people
-- ============================================
CREATE TABLE IF NOT EXISTS faces (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    image_id          INTEGER NOT NULL REFERENCES images(id) ON DELETE CASCADE,
    person_id         INTEGER REFERENCES people(id) ON DELETE SET NULL,

    -- Bounding box (normalized 0.0-1.0 relative to image dimensions)
    bbox_x            REAL NOT NULL,
    bbox_y            REAL NOT NULL,
    bbox_width        REAL NOT NULL,
    bbox_height       REAL NOT NULL,

    -- Detection confidence score
    confidence        REAL NOT NULL,

    -- Face embedding vector (128-dim float32 = 512 bytes)
    embedding         BLOB,

    -- Face crop thumbnail hash
    face_thumbnail    TEXT,

    created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_faces_image_id ON faces(image_id);
CREATE INDEX IF NOT EXISTS idx_faces_person_id ON faces(person_id);

-- ============================================
-- TAGS: User-defined tags
-- ============================================
CREATE TABLE IF NOT EXISTS tags (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT NOT NULL UNIQUE,
    color         TEXT DEFAULT '#888888'
);

-- ============================================
-- IMAGE_TAGS: Many-to-many relationship
-- ============================================
CREATE TABLE IF NOT EXISTS image_tags (
    image_id    INTEGER NOT NULL REFERENCES images(id) ON DELETE CASCADE,
    tag_id      INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (image_id, tag_id)
);

CREATE INDEX IF NOT EXISTS idx_image_tags_tag_id ON image_tags(tag_id);

-- ============================================
-- ALBUMS: Virtual collections
-- ============================================
CREATE TABLE IF NOT EXISTS albums (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT NOT NULL,
    description   TEXT,
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- ============================================
-- ALBUM_IMAGES: Many-to-many relationship
-- ============================================
CREATE TABLE IF NOT EXISTS album_images (
    album_id    INTEGER NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
    image_id    INTEGER NOT NULL REFERENCES images(id) ON DELETE CASCADE,
    PRIMARY KEY (album_id, image_id)
);

CREATE INDEX IF NOT EXISTS idx_album_images_image_id ON album_images(image_id);

-- ============================================
-- PROCESSING_QUEUE: Async background tasks
-- ============================================
CREATE TABLE IF NOT EXISTS processing_queue (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    image_id      INTEGER NOT NULL REFERENCES images(id) ON DELETE CASCADE,
    task_type     TEXT NOT NULL,
    status        TEXT NOT NULL DEFAULT 'pending',
    priority      INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    started_at    TEXT,
    completed_at  TEXT
);

CREATE INDEX IF NOT EXISTS idx_queue_status ON processing_queue(status, priority);
