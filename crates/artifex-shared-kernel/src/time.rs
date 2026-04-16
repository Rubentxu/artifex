//! Time types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// UTC timestamp wrapper for domain events and audit fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    /// Creates a timestamp for the current moment in UTC.
    pub fn now() -> Self {
        Self(Utc::now())
    }

    /// Creates a timestamp from a DateTime<Utc>.
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }

    /// Returns the underlying DateTime<Utc>.
    pub fn into_inner(self) -> DateTime<Utc> {
        self.0
    }

    /// Returns a reference to the underlying DateTime<Utc>.
    pub fn as_datetime(&self) -> &DateTime<Utc> {
        &self.0
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_rfc3339())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_now_returns_recent_utc() {
        let before = Utc::now();
        let ts = Timestamp::now();
        let after = Utc::now();

        assert!(ts.as_datetime() >= &before);
        assert!(ts.as_datetime() <= &after);
    }

    #[test]
    fn test_timestamp_default_is_now() {
        let before = Utc::now();
        let ts = Timestamp::default();
        let after = Utc::now();

        assert!(ts.as_datetime() >= &before);
        assert!(ts.as_datetime() <= &after);
    }

    #[test]
    fn test_timestamp_serde_roundtrip() {
        let ts = Timestamp::now();
        let serialized = serde_json::to_string(&ts).unwrap();
        let deserialized: Timestamp = serde_json::from_str(&serialized).unwrap();
        assert_eq!(ts, deserialized);
    }

    #[test]
    fn test_timestamp_display_format() {
        let ts = Timestamp::now();
        let display = ts.to_string();
        // RFC3339 format should contain 'T' and timezone offset (+00:00 for UTC)
        assert!(display.contains('T'));
        // UTC timestamp should end with +00:00 or Z
        assert!(display.ends_with("+00:00") || display.ends_with('Z'));
    }
}
