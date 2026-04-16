-- Model Config Migration
-- Creates tables for model profiles, routing rules, provider credentials, and prompt templates

-- Model profiles: registered provider/model combinations
CREATE TABLE IF NOT EXISTS model_profiles (
    id TEXT PRIMARY KEY NOT NULL,
    provider_name TEXT NOT NULL,
    model_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    capabilities TEXT NOT NULL DEFAULT '[]',
    enabled INTEGER NOT NULL DEFAULT 1,
    pricing_tier TEXT NOT NULL DEFAULT 'standard',
    config TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Routing rules: operation → primary + fallback profiles
CREATE TABLE IF NOT EXISTS routing_rules (
    id TEXT PRIMARY KEY NOT NULL,
    operation_type TEXT NOT NULL UNIQUE,
    default_profile_id TEXT NOT NULL REFERENCES model_profiles(id),
    fallback_profile_ids TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Provider credentials: keychain reference only (no secret plaintext)
CREATE TABLE IF NOT EXISTS provider_credentials (
    id TEXT PRIMARY KEY NOT NULL,
    provider_name TEXT NOT NULL,
    key_type TEXT NOT NULL DEFAULT 'api_key',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Prompt templates: reusable prompt presets with variables
CREATE TABLE IF NOT EXISTS prompt_templates (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    template_text TEXT NOT NULL,
    variables TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_model_profiles_provider ON model_profiles(provider_name);
CREATE INDEX IF NOT EXISTS idx_model_profiles_enabled ON model_profiles(enabled);
CREATE INDEX IF NOT EXISTS idx_routing_rules_operation ON routing_rules(operation_type);
CREATE INDEX IF NOT EXISTS idx_provider_credentials_name ON provider_credentials(provider_name);
