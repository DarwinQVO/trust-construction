// Trust Construction System - Core Library
// Exposes all modules for use in CLI, API server, and tests

pub mod db;
pub mod parser;
pub mod attributes;     // NEW: Semantic Layer - Attribute Registry
pub mod schema;         // NEW: Shape Layer - Schema Validation
pub mod rules;          // NEW: Classification Rules - Badge 17
pub mod deduplication;  // NEW: Deduplication Engine - Badge 18
pub mod temporal;       // NEW: Temporal Model - Badge 19A
pub mod reconciliation; // NEW: Reconciliation Engine - Badge 19B
pub mod data_quality;   // NEW: Data Quality Engine - Badge 20
pub mod entities;       // NEW: Entity Models - Badge 21

// Re-export commonly used types
pub use db::{
    Transaction, SourceFileStat, Event,
    load_csv, setup_database, insert_transactions,
    get_all_transactions, get_source_file_stats, get_transactions_by_source,
    verify_count, insert_event, get_events_for_entity,
    migrate_add_uuids  // Badge 19: Migration function
};
pub use parser::{
    BankParser, MerchantExtractor, TypeClassifier,
    RawTransaction, SourceType,
    detect_source, get_parser,
    BofAParser, AppleCardParser, StripeParser, WiseParser, ScotiabankParser,
};
pub use attributes::{
    AttributeRegistry, AttributeDefinition, AttributeType, ValidationRule,
};
pub use schema::{
    SchemaValidator, Context, ValidationError, ValidationResult,
};
pub use rules::{
    ClassificationRule, RuleEngine, ClassificationResult,
};
pub use deduplication::{
    DeduplicationEngine, DuplicateMatch, MatchStrategy,
};
pub use temporal::{
    TimeModel, VersionedValue, TemporalEntity, Snapshot,
};
pub use reconciliation::{
    ReconciliationEngine, ReconciliationReport, ReconciliationResult,
    StatementMetadata, Discrepancy, DiscrepancyCategory,
};
pub use data_quality::{
    DataQualityEngine, QualityReport, ValidationResult as QualityValidationResult,
    QualityIssue, Severity, BatchSummary,
};
pub use entities::{
    Bank, BankType, BankRegistry,
    Merchant, MerchantType, MerchantRegistry,
    Category, CategoryType, CategoryRegistry,
    Account, AccountType, AccountRegistry,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Badge progress
pub const BADGES_COMPLETE: u8 = 25;  // Badge 25: Temporal Persistence (ALL 4 ENTITIES) - Rich Hickey 100%! ⏳✅
pub const BADGES_TOTAL: u8 = 25;  // Extended: original 20 + entity models (21-24) + temporal persistence (25) - ALL COMPLETE!

/// Get badge progress as percentage
pub fn badge_progress() -> f32 {
    (BADGES_COMPLETE as f32 / BADGES_TOTAL as f32) * 100.0
}
