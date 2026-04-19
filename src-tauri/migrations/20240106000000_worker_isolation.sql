-- Migration 006: Worker Isolation
-- Adds attempt column to jobs table for retry tracking.

-- Add attempt column to jobs table (defaults to 0 for existing rows)
ALTER TABLE jobs ADD COLUMN attempt INTEGER NOT NULL DEFAULT 0;

-- Backfill: existing jobs that have retries > 0 should have attempt = retries
-- Jobs with NULL retries or 0 retries keep attempt = 0 (their initial state)
UPDATE jobs SET attempt = retries WHERE retries IS NOT NULL AND retries > 0 AND attempt = 0;
