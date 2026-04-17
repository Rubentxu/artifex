//! Routing rule entity.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Maximum number of fallback profiles to try.
pub const MAX_FALLBACK_DEPTH: usize = 3;

/// Represents a routing rule for operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    /// Unique identifier.
    pub id: Uuid,
    /// Operation type (e.g., "imagegen.txt2img", "tts.npc_line").
    pub operation_type: String,
    /// Default profile ID to use.
    pub default_profile_id: Uuid,
    /// Ordered list of fallback profile IDs.
    pub fallback_profile_ids: Vec<Uuid>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
}

impl RoutingRule {
    /// Creates a new routing rule.
    pub fn new(
        operation_type: String,
        default_profile_id: Uuid,
        fallback_profile_ids: Vec<Uuid>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            operation_type,
            default_profile_id,
            fallback_profile_ids,
            created_at: now,
            updated_at: now,
        }
    }

    /// Returns an iterator over all profile IDs to try (default first, then fallbacks).
    pub fn profile_ids(&self) -> impl Iterator<Item = &Uuid> {
        std::iter::once(&self.default_profile_id).chain(self.fallback_profile_ids.iter())
    }

    /// Returns the total number of profiles (default + fallbacks).
    pub fn len(&self) -> usize {
        1 + self.fallback_profile_ids.len()
    }

    /// Returns true if there are no profiles (always false since there's always a default).
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Returns true if there are no fallbacks configured.
    pub fn is_single(&self) -> bool {
        self.fallback_profile_ids.is_empty()
    }

    /// Validates the routing rule.
    pub fn validate(&self) -> Result<(), String> {
        if self.operation_type.is_empty() {
            return Err("Operation type cannot be empty".to_string());
        }
        if self.fallback_profile_ids.len() > MAX_FALLBACK_DEPTH {
            return Err(format!(
                "Maximum {} fallbacks allowed, got {}",
                MAX_FALLBACK_DEPTH,
                self.fallback_profile_ids.len()
            ));
        }
        // Check for duplicates
        let all_ids: Vec<_> = self.profile_ids().collect();
        if all_ids.len()
            != all_ids
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
        {
            return Err("Duplicate profile IDs in routing rule".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing_rule_new() {
        let default_id = Uuid::new_v4();
        let fallback_id = Uuid::new_v4();

        let rule = RoutingRule::new(
            "imagegen.txt2img".to_string(),
            default_id,
            vec![fallback_id],
        );

        assert!(!rule.id.is_nil());
        assert_eq!(rule.operation_type, "imagegen.txt2img");
        assert_eq!(rule.default_profile_id, default_id);
        assert_eq!(rule.fallback_profile_ids.len(), 1);
    }

    #[test]
    fn test_routing_rule_profile_ids() {
        let default_id = Uuid::new_v4();
        let fallback1 = Uuid::new_v4();
        let fallback2 = Uuid::new_v4();

        let rule = RoutingRule::new("test".to_string(), default_id, vec![fallback1, fallback2]);

        let ids: Vec<_> = rule.profile_ids().collect();
        assert_eq!(ids.len(), 3);
        assert_eq!(ids[0], &default_id);
        assert_eq!(ids[1], &fallback1);
        assert_eq!(ids[2], &fallback2);
    }

    #[test]
    fn test_routing_rule_validate_empty_operation() {
        let rule = RoutingRule::new("".to_string(), Uuid::new_v4(), vec![]);
        assert!(rule.validate().is_err());
        assert!(rule.validate().unwrap_err().contains("empty"));
    }

    #[test]
    fn test_routing_rule_validate_too_many_fallbacks() {
        let fallbacks: Vec<_> = (0..5).map(|_| Uuid::new_v4()).collect();
        let rule = RoutingRule::new("test".to_string(), Uuid::new_v4(), fallbacks);
        assert!(rule.validate().is_err());
        assert!(rule.validate().unwrap_err().contains("3"));
    }

    #[test]
    fn test_routing_rule_validate_duplicate_ids() {
        let id = Uuid::new_v4();
        let rule = RoutingRule::new(
            "test".to_string(),
            id,
            vec![id], // Same as default
        );
        assert!(rule.validate().is_err());
        assert!(rule.validate().unwrap_err().contains("Duplicate"));
    }
}
