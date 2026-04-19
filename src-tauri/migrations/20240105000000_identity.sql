-- Migration 005: Identity and Usage Tracking
-- Creates user_profile (single-row) and usage_counters tables for tier/quota management

-- User profile table (single-row, seeded on first launch)
CREATE TABLE IF NOT EXISTS user_profile (
    id TEXT PRIMARY KEY NOT NULL,
    display_name TEXT NOT NULL DEFAULT '',
    email TEXT,
    avatar_path TEXT,
    tier TEXT NOT NULL DEFAULT 'free' CHECK (tier IN ('free', 'pro')),
    license_key TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Usage counters table (per-operation, per-month)
CREATE TABLE IF NOT EXISTS usage_counters (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL REFERENCES user_profile(id),
    operation_type TEXT NOT NULL,
    period TEXT NOT NULL,
    count INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL
);

-- Unique index on (user_id, operation_type, period) for upsert behavior
CREATE UNIQUE INDEX IF NOT EXISTS idx_usage_counters_unique ON usage_counters(user_id, operation_type, period);

-- Seed default user profile (idempotent via INSERT OR IGNORE)
INSERT OR IGNORE INTO user_profile (id, tier, created_at, updated_at)
VALUES ('default-user', 'free', datetime('now'), datetime('now'));
