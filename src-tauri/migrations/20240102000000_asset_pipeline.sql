-- Expand jobs table for asset pipeline
ALTER TABLE jobs ADD COLUMN operation TEXT;
ALTER TABLE jobs ADD COLUMN progress_percent INTEGER NOT NULL DEFAULT 0;
ALTER TABLE jobs ADD COLUMN progress_message TEXT;
ALTER TABLE jobs ADD COLUMN error_message TEXT;
ALTER TABLE jobs ADD COLUMN started_at TEXT;
ALTER TABLE jobs ADD COLUMN completed_at TEXT;

-- Expand assets table for asset pipeline
ALTER TABLE assets ADD COLUMN file_path TEXT;
ALTER TABLE assets ADD COLUMN metadata TEXT;
ALTER TABLE assets ADD COLUMN file_size INTEGER;
ALTER TABLE assets ADD COLUMN width INTEGER;
ALTER TABLE assets ADD COLUMN height INTEGER;

-- Indexes for common query patterns
CREATE INDEX idx_jobs_project_status ON jobs(project_id, status);
CREATE INDEX idx_assets_project_kind ON assets(project_id, kind);
