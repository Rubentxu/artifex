//! Domain event trait.

use uuid::Uuid;

use crate::time::Timestamp;

/// Trait for domain events that can be published and recorded.
pub trait DomainEvent: Send + Sync {
    /// Returns the unique event identifier.
    fn event_id(&self) -> Uuid;

    /// Returns the timestamp when this event occurred.
    fn occurred_at(&self) -> Timestamp;

    /// Returns the event type name for discrimination.
    fn event_type(&self) -> &'static str;
}
