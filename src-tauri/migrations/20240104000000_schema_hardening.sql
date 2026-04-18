PRAGMA foreign_keys = OFF;

-- projects: add doc-required settings without breaking current code
ALTER TABLE projects ADD COLUMN settings JSON NOT NULL DEFAULT '{}';

-- collections: new grouping table from master document
CREATE TABLE IF NOT EXISTS collections (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_collections_project ON collections(project_id);

-- assets: add missing doc-required columns, keep existing pipeline columns
ALTER TABLE assets ADD COLUMN tags JSON NOT NULL DEFAULT '[]';
ALTER TABLE assets ADD COLUMN import_source TEXT NOT NULL DEFAULT 'uploaded';
ALTER TABLE assets ADD COLUMN collection_id TEXT REFERENCES collections(id);
CREATE INDEX IF NOT EXISTS idx_assets_project ON assets(project_id);
CREATE INDEX IF NOT EXISTS idx_assets_collection ON assets(collection_id);

-- asset_versions: rebuild now because code does not use this table yet
ALTER TABLE asset_versions RENAME TO asset_versions_legacy;

CREATE TABLE asset_versions (
    id TEXT PRIMARY KEY NOT NULL,
    asset_id TEXT NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    storage_path TEXT NOT NULL,
    checksum TEXT NOT NULL,
    size_bytes INTEGER,
    width_px INTEGER,
    height_px INTEGER,
    duration_ms INTEGER,
    format TEXT,
    derived_from TEXT REFERENCES asset_versions(id),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO asset_versions (
    id,
    asset_id,
    storage_path,
    checksum,
    created_at
)
SELECT
    id,
    asset_id,
    file_path,
    COALESCE(checksum, ''),
    created_at
FROM asset_versions_legacy;

DROP TABLE asset_versions_legacy;
CREATE INDEX IF NOT EXISTS idx_asset_versions_asset ON asset_versions(asset_id);

-- jobs: add doc-required scheduler/result columns without removing legacy runtime columns
ALTER TABLE jobs ADD COLUMN priority INTEGER NOT NULL DEFAULT 50;
ALTER TABLE jobs ADD COLUMN worker_kind TEXT;
ALTER TABLE jobs ADD COLUMN params JSON NOT NULL DEFAULT '{}';
ALTER TABLE jobs ADD COLUMN result JSON;
ALTER TABLE jobs ADD COLUMN error TEXT;
ALTER TABLE jobs ADD COLUMN retries INTEGER NOT NULL DEFAULT 0;
ALTER TABLE jobs ADD COLUMN max_retries INTEGER NOT NULL DEFAULT 2;
ALTER TABLE jobs ADD COLUMN dependencies JSON NOT NULL DEFAULT '[]';
ALTER TABLE jobs ADD COLUMN worker_id TEXT;
ALTER TABLE jobs ADD COLUMN submitted_at TEXT;

UPDATE jobs
SET worker_kind = COALESCE(worker_kind, job_type),
    params = CASE
        WHEN params IS NULL OR params = '' THEN COALESCE(operation, '{}')
        ELSE params
    END,
    error = COALESCE(error, error_message),
    submitted_at = COALESCE(submitted_at, created_at);

CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status, worker_kind);
CREATE INDEX IF NOT EXISTS idx_jobs_submitted ON jobs(submitted_at DESC);

PRAGMA foreign_keys = ON;
