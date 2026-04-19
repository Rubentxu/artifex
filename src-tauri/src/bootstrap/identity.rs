//! Bootstrap for identity service.

use std::sync::Arc;

use sqlx::SqlitePool;

use crate::identity::repository::create_identity_repository;
use crate::identity::service::IdentityService;

/// Creates the identity service and seeds default data (synchronous, for use in setup).
pub fn create_identity_service(
    pool: &SqlitePool,
) -> Result<Arc<IdentityService>, String> {
    let repo = create_identity_repository(pool.clone());
    let service = Arc::new(IdentityService::new(repo));

    // Seed identity data synchronously using block_in_place
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(seed_identity(&*service))
    }).map_err(|e| format!("Failed to seed identity: {}", e))?;

    Ok(service)
}

/// Seeds the identity data (called on every service init for idempotency).
async fn seed_identity(service: &IdentityService) -> Result<(), String> {
    // Ensure a profile exists (should always exist after migration, but double-check)
    match service.get_profile().await {
        Ok(_) => {} // Profile exists, all good
        Err(_) => {
            // Profile doesn't exist, create default
            service
                .update_profile(
                    Some("Default User".to_string()),
                    None,
                    None,
                )
                .await
                .map_err(|e| format!("Failed to seed user profile: {}", e))?;
        }
    }
    Ok(())
}
