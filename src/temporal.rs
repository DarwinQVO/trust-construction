// ⏰ Temporal Model - Badge 19
// Implements Rich Hickey's philosophy: "Time must be explicit"
//
// Four distinct times:
// 1. Business Time: When the real-world event occurred
// 2. System Time: When we learned about it (ingestion)
// 3. Valid Time: When this value was/is true
// 4. Decision Time: When actions were taken on it

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// TIME MODEL
// ============================================================================

/// TimeModel - Four distinct times for complete temporal tracking
///
/// Following Rich Hickey's "Value of Values" talk:
/// - "Time is not a single thing"
/// - "Different times for different purposes"
/// - "Must be explicit to enable temporal queries"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeModel {
    // ========================================================================
    // 1. BUSINESS TIME (Real-World Event Time)
    // ========================================================================
    /// When the transaction actually occurred in the real world
    /// Example: "12/31/2024" - The date on the bank statement
    /// This is the ONLY time that matters for financial reports
    pub business_time: String,

    // ========================================================================
    // 2. SYSTEM TIME (Ingestion Time)
    // ========================================================================
    /// When this record was created in our system
    /// Example: "2025-01-05T10:30:00Z" - When we imported the CSV
    /// Used for: Audit trail, tracking pipeline delays
    pub system_time: DateTime<Utc>,

    // ========================================================================
    // 3. VALID TIME (Truth Time Range)
    // ========================================================================
    /// When this particular VALUE became true
    /// Example: Transaction imported with category="Unknown" at T1
    ///          User corrects to category="Restaurants" at T2
    ///          → Two versions, each with different valid_from
    pub valid_from: DateTime<Utc>,

    /// When this VALUE ceased to be true (None = still current)
    /// Example: When user corrected category, old version gets valid_until=T2
    pub valid_until: Option<DateTime<Utc>>,

    // ========================================================================
    // 4. DECISION TIME (Action Time)
    // ========================================================================
    /// When classification decision was made
    pub classified_at: Option<DateTime<Utc>>,

    /// When human verification was done
    pub verified_at: Option<DateTime<Utc>>,

    /// When this record was marked for review
    pub flagged_at: Option<DateTime<Utc>>,
}

impl TimeModel {
    /// Create new time model for freshly imported transaction
    pub fn new(business_time: String) -> Self {
        let now = Utc::now();
        TimeModel {
            business_time,
            system_time: now,
            valid_from: now,
            valid_until: None, // Still current
            classified_at: None,
            verified_at: None,
            flagged_at: None,
        }
    }

    /// Check if this version is current (no valid_until)
    pub fn is_current(&self) -> bool {
        self.valid_until.is_none()
    }

    /// Check if this version was valid at a specific time
    pub fn was_valid_at(&self, time: DateTime<Utc>) -> bool {
        self.valid_from <= time && self.valid_until.map_or(true, |until| until > time)
    }

    /// Close this version (set valid_until)
    pub fn close(&mut self) {
        self.valid_until = Some(Utc::now());
    }

    /// Mark as classified
    pub fn mark_classified(&mut self) {
        self.classified_at = Some(Utc::now());
    }

    /// Mark as verified
    pub fn mark_verified(&mut self) {
        self.verified_at = Some(Utc::now());
    }

    /// Mark as flagged for review
    pub fn mark_flagged(&mut self) {
        self.flagged_at = Some(Utc::now());
    }
}

// ============================================================================
// VERSIONED VALUE
// ============================================================================

/// VersionedValue - Wraps any value with temporal metadata
///
/// Following Rich Hickey: "Values are immutable. Identity has many values over time."
///
/// Example:
/// ```
/// Transaction #123 (identity) has these values over time:
///   Version 1 (valid 2025-01-01 → 2025-01-15): category="Unknown"
///   Version 2 (valid 2025-01-15 → now):        category="Restaurants"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedValue<T> {
    /// The immutable value (snapshot)
    pub value: T,

    /// Version number (monotonically increasing)
    pub version: i64,

    /// Temporal metadata
    pub time: TimeModel,

    /// Who created this version
    pub created_by: String,

    /// Why this version was created
    pub change_reason: Option<String>,
}

impl<T> VersionedValue<T> {
    /// Create new versioned value
    pub fn new(value: T, business_time: String, created_by: String) -> Self {
        VersionedValue {
            value,
            version: 1,
            time: TimeModel::new(business_time),
            created_by,
            change_reason: None,
        }
    }

    /// Create new version from previous
    pub fn next_version(
        &self,
        new_value: T,
        actor: String,
        reason: Option<String>,
    ) -> VersionedValue<T> {
        let now = Utc::now();
        VersionedValue {
            value: new_value,
            version: self.version + 1,
            time: TimeModel {
                business_time: self.time.business_time.clone(),
                system_time: self.time.system_time, // Inherited
                valid_from: now,
                valid_until: None,
                classified_at: self.time.classified_at,
                verified_at: None, // Reset verification
                flagged_at: self.time.flagged_at,
            },
            created_by: actor,
            change_reason: reason,
        }
    }

    /// Check if this version is current
    pub fn is_current(&self) -> bool {
        self.time.is_current()
    }

    /// Check if this version was valid at specific time
    pub fn was_valid_at(&self, time: DateTime<Utc>) -> bool {
        self.time.was_valid_at(time)
    }
}

// ============================================================================
// TEMPORAL ENTITY
// ============================================================================

/// TemporalEntity - Identity + Timeline of values
///
/// Following Rich Hickey: "Identity persists. Values change."
///
/// An entity has:
/// - Stable identity (UUID) that never changes
/// - Timeline of immutable values
/// - Each value has temporal metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalEntity<T> {
    /// Stable identity (UUID - never changes)
    pub id: String,

    /// Timeline of immutable values (append-only)
    pub versions: Vec<VersionedValue<T>>,
}

impl<T: Clone> TemporalEntity<T> {
    /// Create new entity with initial value
    pub fn new(id: String, initial_value: T, business_time: String, creator: String) -> Self {
        TemporalEntity {
            id,
            versions: vec![VersionedValue::new(initial_value, business_time, creator)],
        }
    }

    /// Get current value (latest version)
    pub fn current(&self) -> Option<&VersionedValue<T>> {
        self.versions.last()
    }

    /// Get current value (mutable)
    pub fn current_mut(&mut self) -> Option<&mut VersionedValue<T>> {
        self.versions.last_mut()
    }

    /// Get value at specific version number
    pub fn at_version(&self, version: i64) -> Option<&VersionedValue<T>> {
        self.versions.iter().find(|v| v.version == version)
    }

    /// Get value as it was at specific time
    pub fn as_of(&self, time: DateTime<Utc>) -> Option<&VersionedValue<T>> {
        self.versions
            .iter()
            .find(|v| v.was_valid_at(time))
    }

    /// Get complete history (all versions)
    pub fn history(&self) -> &[VersionedValue<T>] {
        &self.versions
    }

    /// Add new version (closes previous version)
    pub fn update(
        &mut self,
        new_value: T,
        actor: String,
        reason: Option<String>,
    ) -> Result<i64, String> {
        // Close current version
        if let Some(current) = self.current_mut() {
            current.time.close();
        }

        // Create next version
        let next = if let Some(current) = self.current() {
            current.next_version(new_value, actor, reason)
        } else {
            return Err("No current version to update from".to_string());
        };

        let version_num = next.version;
        self.versions.push(next);

        Ok(version_num)
    }

    /// Count total versions
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }

    /// Check if entity has multiple versions
    pub fn has_history(&self) -> bool {
        self.versions.len() > 1
    }
}

// ============================================================================
// SNAPSHOT
// ============================================================================

/// Snapshot - Immutable view of multiple entities at specific time
///
/// Following Rich Hickey: "Snapshot = consistent view at point in time"
///
/// Use case: "Show me all transactions as they were on 2024-12-31"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot<T> {
    /// Unique snapshot ID
    pub snapshot_id: String,

    /// Point in time this snapshot represents
    pub as_of: DateTime<Utc>,

    /// Who created this snapshot
    pub created_by: String,

    /// Optional label
    pub label: Option<String>,

    /// Immutable values at this time
    pub values: Vec<T>,

    /// Metadata
    pub metadata: serde_json::Value,
}

impl<T> Snapshot<T> {
    /// Create new snapshot
    pub fn new(
        as_of: DateTime<Utc>,
        creator: String,
        label: Option<String>,
        values: Vec<T>,
        metadata: serde_json::Value,
    ) -> Self {
        Snapshot {
            snapshot_id: uuid::Uuid::new_v4().to_string(),
            as_of,
            created_by: creator,
            label,
            values,
            metadata,
        }
    }

    /// Count values in snapshot
    pub fn count(&self) -> usize {
        self.values.len()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestValue {
        category: String,
        confidence: f64,
    }

    #[test]
    fn test_time_model_creation() {
        let time = TimeModel::new("12/31/2024".to_string());

        assert_eq!(time.business_time, "12/31/2024");
        assert!(time.is_current());
        assert!(time.classified_at.is_none());
        assert!(time.verified_at.is_none());
    }

    #[test]
    fn test_time_model_validity() {
        let mut time = TimeModel::new("12/31/2024".to_string());
        let t1 = Utc::now();

        std::thread::sleep(std::time::Duration::from_millis(10));
        time.close();
        let t2 = Utc::now();

        // Should be valid at t1, not at t2
        assert!(time.was_valid_at(t1));
        assert!(!time.was_valid_at(t2));
        assert!(!time.is_current());
    }

    #[test]
    fn test_versioned_value_creation() {
        let value = TestValue {
            category: "Unknown".to_string(),
            confidence: 0.5,
        };

        let versioned = VersionedValue::new(value, "12/31/2024".to_string(), "importer".to_string());

        assert_eq!(versioned.version, 1);
        assert_eq!(versioned.value.category, "Unknown");
        assert!(versioned.is_current());
    }

    #[test]
    fn test_versioned_value_next_version() {
        let v1_value = TestValue {
            category: "Unknown".to_string(),
            confidence: 0.5,
        };

        let v1 = VersionedValue::new(v1_value, "12/31/2024".to_string(), "importer".to_string());

        let v2_value = TestValue {
            category: "Restaurants".to_string(),
            confidence: 0.95,
        };

        let v2 = v1.next_version(v2_value, "user_123".to_string(), Some("Manual correction".to_string()));

        assert_eq!(v2.version, 2);
        assert_eq!(v2.value.category, "Restaurants");
        assert_eq!(v2.created_by, "user_123");
        assert_eq!(v2.change_reason, Some("Manual correction".to_string()));
    }

    #[test]
    fn test_temporal_entity_creation() {
        let value = TestValue {
            category: "Unknown".to_string(),
            confidence: 0.5,
        };

        let entity = TemporalEntity::new(
            "tx-123".to_string(),
            value,
            "12/31/2024".to_string(),
            "importer".to_string(),
        );

        assert_eq!(entity.id, "tx-123");
        assert_eq!(entity.version_count(), 1);
        assert!(!entity.has_history());

        let current = entity.current().unwrap();
        assert_eq!(current.version, 1);
        assert_eq!(current.value.category, "Unknown");
    }

    #[test]
    fn test_temporal_entity_update() {
        let initial = TestValue {
            category: "Unknown".to_string(),
            confidence: 0.5,
        };

        let mut entity = TemporalEntity::new(
            "tx-123".to_string(),
            initial,
            "12/31/2024".to_string(),
            "importer".to_string(),
        );

        let updated = TestValue {
            category: "Restaurants".to_string(),
            confidence: 0.95,
        };

        let new_version = entity
            .update(updated, "user_123".to_string(), Some("Corrected by user".to_string()))
            .unwrap();

        assert_eq!(new_version, 2);
        assert_eq!(entity.version_count(), 2);
        assert!(entity.has_history());

        // Check current value
        let current = entity.current().unwrap();
        assert_eq!(current.version, 2);
        assert_eq!(current.value.category, "Restaurants");

        // Check previous version is closed
        let v1 = entity.at_version(1).unwrap();
        assert!(!v1.is_current());
        assert!(v1.time.valid_until.is_some());
    }

    #[test]
    fn test_temporal_entity_as_of() {
        let initial = TestValue {
            category: "Unknown".to_string(),
            confidence: 0.5,
        };

        let mut entity = TemporalEntity::new(
            "tx-123".to_string(),
            initial,
            "12/31/2024".to_string(),
            "importer".to_string(),
        );

        let t1 = Utc::now();

        std::thread::sleep(std::time::Duration::from_millis(10));

        let updated = TestValue {
            category: "Restaurants".to_string(),
            confidence: 0.95,
        };

        entity
            .update(updated, "user_123".to_string(), None)
            .unwrap();

        let t2 = Utc::now();

        // At t1, should get version 1
        let v_at_t1 = entity.as_of(t1).unwrap();
        assert_eq!(v_at_t1.version, 1);
        assert_eq!(v_at_t1.value.category, "Unknown");

        // At t2, should get version 2
        let v_at_t2 = entity.as_of(t2).unwrap();
        assert_eq!(v_at_t2.version, 2);
        assert_eq!(v_at_t2.value.category, "Restaurants");
    }

    #[test]
    fn test_snapshot_creation() {
        let values = vec![
            TestValue {
                category: "Food".to_string(),
                confidence: 0.9,
            },
            TestValue {
                category: "Transport".to_string(),
                confidence: 0.85,
            },
        ];

        let snapshot = Snapshot::new(
            Utc::now(),
            "user_123".to_string(),
            Some("December 2024 close".to_string()),
            values,
            serde_json::json!({"total": 2}),
        );

        assert_eq!(snapshot.count(), 2);
        assert_eq!(snapshot.label, Some("December 2024 close".to_string()));
        assert_eq!(snapshot.created_by, "user_123");
    }
}
