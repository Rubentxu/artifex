-- Migration: Asset Lineage
-- Adds derived_from_asset_id column to assets table for lineage tracking.
-- The collections table and tags/import_source columns already exist from migration 004.

ALTER TABLE assets ADD COLUMN derived_from_asset_id TEXT REFERENCES assets(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_assets_derived_from ON assets(derived_from_asset_id);
