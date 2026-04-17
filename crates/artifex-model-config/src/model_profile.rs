//! Model profile entity.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::provider::ModelCapability;

/// Pricing tier for a model profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PricingTier {
    Free,
    Standard,
    Premium,
}

impl PricingTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            PricingTier::Free => "free",
            PricingTier::Standard => "standard",
            PricingTier::Premium => "premium",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "free" => Some(Self::Free),
            "standard" => Some(Self::Standard),
            "premium" => Some(Self::Premium),
            _ => None,
        }
    }
}

/// Represents a model profile configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfile {
    /// Unique identifier.
    pub id: Uuid,
    /// Provider name (e.g., "replicate", "fal").
    pub provider_name: String,
    /// Provider's model identifier.
    pub model_id: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Capabilities supported by this model.
    pub capabilities: Vec<ModelCapability>,
    /// Whether this profile is enabled.
    pub enabled: bool,
    /// Pricing tier.
    pub pricing_tier: PricingTier,
    /// Provider-specific configuration as JSON.
    pub config: serde_json::Value,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
}

impl ModelProfile {
    /// Creates a new model profile.
    pub fn new(
        provider_name: String,
        model_id: String,
        display_name: String,
        capabilities: Vec<ModelCapability>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            provider_name,
            model_id,
            display_name,
            capabilities,
            enabled: true,
            pricing_tier: PricingTier::Standard,
            config: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        }
    }

    /// Returns true if the profile supports the given capability.
    pub fn supports(&self, capability: ModelCapability) -> bool {
        self.capabilities.contains(&capability)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_profile_new() {
        let profile = ModelProfile::new(
            "replicate".to_string(),
            "black-forest-labs/flux-1.1-pro".to_string(),
            "FLUX 1.1 Pro".to_string(),
            vec![ModelCapability::ImageGen],
        );

        assert!(!profile.id.is_nil());
        assert_eq!(profile.provider_name, "replicate");
        assert_eq!(profile.model_id, "black-forest-labs/flux-1.1-pro");
        assert_eq!(profile.display_name, "FLUX 1.1 Pro");
        assert!(profile.enabled);
        assert!(profile.supports(ModelCapability::ImageGen));
        assert!(!profile.supports(ModelCapability::TextComplete));
    }

    #[test]
    fn test_pricing_tier() {
        assert_eq!(PricingTier::Free.as_str(), "free");
        assert_eq!(
            PricingTier::from_str("standard"),
            Some(PricingTier::Standard)
        );
        assert_eq!(PricingTier::from_str("unknown"), None);
    }
}
