//! Model configuration module.
//!
//! This module provides model profile management, routing rules,
//! prompt templates, and provider adapters for AI services.

pub mod commands;
pub mod keychain_store;
pub mod providers;
pub mod repository;
pub mod service;

pub use commands::{
    create_model_profile, create_prompt_template, delete_credential,
    delete_model_profile, delete_prompt_template, get_credential_status,
    get_provider, list_model_profiles, list_providers, list_prompt_templates,
    list_routing_rules, set_credential, set_provider_enabled, set_routing_rule, test_provider_connection,
    update_model_profile,
};
pub use providers::{
    FalImageProvider, HuggingFaceImageProvider, ReplicateImageProvider,
    TogetherTextProvider, KieImageProvider,
};
pub use repository::{
    create_profile, create_rule, create_template, delete_profile, delete_rule,
    delete_template, delete_credential_ref, find_profile, find_rule, find_template,
    list_credential_refs, list_profiles, list_rules, list_templates, update_profile,
    update_rule, update_template, seed_defaults, SqliteModelConfigRepository,
};
pub use keychain_store::KeychainCredentialStore;
pub use service::{CredentialStatus, ModelConfigService, register_builtin_providers};