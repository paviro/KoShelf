CREATE TABLE IF NOT EXISTS library_items (
    id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL UNIQUE,
    format TEXT NOT NULL CHECK (format IN ('epub', 'fb2', 'mobi', 'cbz', 'cbr')),
    content_type TEXT NOT NULL CHECK (content_type IN ('book', 'comic')),
    title TEXT NOT NULL,
    authors_json TEXT NOT NULL DEFAULT '[]',
    series_json TEXT,
    description TEXT,
    language TEXT,
    publisher TEXT,
    subjects_json TEXT NOT NULL DEFAULT '[]',
    identifiers_json TEXT NOT NULL DEFAULT '[]',
    status TEXT NOT NULL CHECK (status IN ('reading', 'complete', 'abandoned', 'unknown')),
    progress_percentage REAL,
    rating INTEGER CHECK (rating IS NULL OR (rating >= 1 AND rating <= 5)),
    review_note TEXT,
    doc_pages INTEGER CHECK (doc_pages IS NULL OR doc_pages > 0),
    pagemap_doc_pages INTEGER CHECK (pagemap_doc_pages IS NULL OR pagemap_doc_pages > 0),
    has_synthetic_pagination INTEGER NOT NULL DEFAULT 0,
    parser_pages INTEGER CHECK (parser_pages IS NULL OR parser_pages > 0),
    hidden_flow_pages INTEGER CHECK (hidden_flow_pages IS NULL OR hidden_flow_pages > 0),
    cover_url TEXT NOT NULL,
    search_base_path TEXT NOT NULL,
    annotation_count INTEGER NOT NULL DEFAULT 0 CHECK (annotation_count >= 0),
    bookmark_count INTEGER NOT NULL DEFAULT 0 CHECK (bookmark_count >= 0),
    highlight_count INTEGER NOT NULL DEFAULT 0 CHECK (highlight_count >= 0),
    partial_md5_checksum TEXT,
    reader_presentation JSON,
    chapters_json TEXT NOT NULL DEFAULT '[]',
    last_open_at TEXT,
    total_reading_time_sec INTEGER CHECK (total_reading_time_sec IS NULL OR total_reading_time_sec >= 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS library_annotations (
    id TEXT PRIMARY KEY,
    item_id TEXT NOT NULL,
    annotation_kind TEXT NOT NULL CHECK (annotation_kind IN ('highlight', 'bookmark')),
    lua_index INTEGER NOT NULL CHECK (lua_index >= 0),
    chapter TEXT,
    datetime TEXT,
    datetime_updated TEXT,
    pageno INTEGER CHECK (pageno IS NULL OR pageno >= 0),
    text TEXT,
    note TEXT,
    pos0 TEXT,
    pos1 TEXT,
    color TEXT,
    drawer TEXT,
    FOREIGN KEY (item_id) REFERENCES library_items(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS library_item_fingerprints (
    item_id TEXT PRIMARY KEY,
    book_path TEXT NOT NULL,
    book_size_bytes INTEGER NOT NULL CHECK (book_size_bytes >= 0),
    book_modified_unix_ms INTEGER NOT NULL CHECK (book_modified_unix_ms >= 0),
    metadata_path TEXT,
    metadata_size_bytes INTEGER CHECK (metadata_size_bytes IS NULL OR metadata_size_bytes >= 0),
    metadata_modified_unix_ms INTEGER CHECK (metadata_modified_unix_ms IS NULL OR metadata_modified_unix_ms >= 0),
    updated_at TEXT NOT NULL,
    FOREIGN KEY (item_id) REFERENCES library_items(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS library_collision_diagnostics (
    canonical_id TEXT NOT NULL,
    file_path TEXT NOT NULL,
    winner_item_id TEXT NOT NULL,
    reason TEXT NOT NULL,
    detected_at TEXT NOT NULL,
    PRIMARY KEY (canonical_id, file_path),
    FOREIGN KEY (winner_item_id) REFERENCES library_items(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_library_items_scope_status
    ON library_items (content_type, status, id);

CREATE INDEX IF NOT EXISTS idx_library_items_scope_progress
    ON library_items (content_type, progress_percentage, id);

CREATE INDEX IF NOT EXISTS idx_library_items_scope_rating
    ON library_items (content_type, rating, id);

CREATE INDEX IF NOT EXISTS idx_library_items_scope_annotations
    ON library_items (content_type, annotation_count, id);

CREATE INDEX IF NOT EXISTS idx_library_items_scope_last_open_at
    ON library_items (content_type, last_open_at, id);

CREATE INDEX IF NOT EXISTS idx_library_items_partial_md5_checksum
    ON library_items (partial_md5_checksum);

CREATE UNIQUE INDEX IF NOT EXISTS idx_library_annotations_item_lua_index
    ON library_annotations (item_id, lua_index);

CREATE UNIQUE INDEX IF NOT EXISTS idx_library_item_fingerprints_book_path
    ON library_item_fingerprints (book_path);

CREATE INDEX IF NOT EXISTS idx_library_item_fingerprints_metadata_path
    ON library_item_fingerprints (metadata_path);

CREATE INDEX IF NOT EXISTS idx_library_collision_diagnostics_winner_item_id
    ON library_collision_diagnostics (winner_item_id);

CREATE TABLE IF NOT EXISTS share_image_fingerprints (
    year INTEGER PRIMARY KEY,
    fingerprint TEXT NOT NULL
);
