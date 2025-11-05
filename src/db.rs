use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

/// Transaction with extensible metadata
/// Core fields are immutable, metadata can grow without breaking changes
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Transaction {
    // ========================================================================
    // CORE FIELDS (never change - immutable schema)
    // ========================================================================
    #[serde(rename = "Date")]
    pub date: String,

    #[serde(rename = "Description")]
    pub description: String,

    #[serde(rename = "Amount_Original")]
    pub amount_original: String,

    #[serde(rename = "Amount_Numeric")]
    pub amount_numeric: f64,

    #[serde(rename = "Transaction_Type")]
    pub transaction_type: String,

    #[serde(rename = "Category")]
    pub category: String,

    #[serde(rename = "Merchant")]
    pub merchant: String,

    #[serde(rename = "Currency")]
    pub currency: String,

    #[serde(rename = "Account_Name")]
    pub account_name: String,

    #[serde(rename = "Account_Number")]
    pub account_number: String,

    #[serde(rename = "Bank")]
    pub bank: String,

    #[serde(rename = "Source_File")]
    pub source_file: String,

    #[serde(rename = "Line_Number")]
    pub line_number: String,

    #[serde(rename = "Classification_Notes")]
    pub classification_notes: String,

    // ========================================================================
    // IDENTITY & VERSIONING (Badge 19 - Rich Hickey's Identity/Value/State)
    // ========================================================================
    /// Stable identity (UUID) - NEVER changes, even when values are corrected
    /// This is DIFFERENT from idempotency_hash (which is for deduplication)
    #[serde(default = "default_uuid")]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,

    /// Version number (monotonically increasing)
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub version: i64,

    // ========================================================================
    // TIME MODEL (Badge 19 - Make time explicit)
    // ========================================================================
    /// System time: When this record was created in our system
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_time: Option<DateTime<Utc>>,

    /// Valid from: When this value became true
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<DateTime<Utc>>,

    /// Valid until: When this value ceased to be true (None = still current)
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>,

    /// Previous version ID: Link to previous version (for temporal queries)
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_version_id: Option<String>,

    // ========================================================================
    // EXTENSIBLE METADATA (can grow without schema changes)
    // Following Rich Hickey's philosophy: "Aggregates as maps, not structs"
    // ========================================================================
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

// Helper functions for serde defaults
fn default_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn is_zero_i64(val: &i64) -> bool {
    *val == 0
}

impl Transaction {
    /// Compute idempotency hash for duplicate detection
    /// NOTE: This is for DEDUPLICATION, not IDENTITY!
    /// Identity = id (UUID), Deduplication = hash
    pub fn compute_idempotency_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!(
            "{}{}{}{}",
            self.date, self.amount_numeric, self.merchant, self.bank
        ));
        format!("{:x}", hasher.finalize())
    }

    // ========================================================================
    // VERSIONING HELPERS (Badge 19 - Rich Hickey's Identity/Value/State)
    // ========================================================================

    /// Initialize temporal fields for a new transaction
    pub fn init_temporal_fields(&mut self) {
        let now = Utc::now();

        // Set UUID if not present
        if self.id.is_empty() {
            self.id = uuid::Uuid::new_v4().to_string();
        }

        // Set version to 1 if 0
        if self.version == 0 {
            self.version = 1;
        }

        // Set timestamps
        if self.system_time.is_none() {
            self.system_time = Some(now);
        }
        if self.valid_from.is_none() {
            self.valid_from = Some(now);
        }
    }

    /// Check if this transaction is current (no valid_until)
    pub fn is_current(&self) -> bool {
        self.valid_until.is_none()
    }

    /// Check if this transaction was valid at specific time
    pub fn was_valid_at(&self, time: DateTime<Utc>) -> bool {
        if let Some(valid_from) = self.valid_from {
            if valid_from > time {
                return false;
            }
        }

        if let Some(valid_until) = self.valid_until {
            if valid_until <= time {
                return false;
            }
        }

        true
    }

    /// Close this version (set valid_until to now)
    pub fn close_version(&mut self) {
        self.valid_until = Some(Utc::now());
    }

    /// Create next version from this transaction
    /// Increments version, updates timestamps, preserves identity
    pub fn next_version(&self, change_reason: Option<String>) -> Transaction {
        let now = Utc::now();

        let mut next = self.clone();
        next.version += 1;
        next.valid_from = Some(now);
        next.valid_until = None;  // New version is current
        next.previous_version_id = Some(self.id.clone());

        // Store change reason in metadata
        if let Some(reason) = change_reason {
            next.metadata.insert(
                "change_reason".to_string(),
                serde_json::json!(reason),
            );
        }

        next
    }

    /// Get identity (stable UUID)
    pub fn identity(&self) -> &str {
        &self.id
    }

    /// Get version number
    pub fn get_version(&self) -> i64 {
        self.version
    }

    // ========================================================================
    // EXTENSIBILITY HELPERS
    // Add new fields without modifying struct or database schema
    // ========================================================================

    /// Set provenance metadata (when and how this transaction was extracted)
    pub fn set_provenance(
        &mut self,
        extracted_at: DateTime<Utc>,
        parser_version: &str,
        transformation_log: Vec<String>,
    ) {
        self.metadata.insert(
            "extracted_at".to_string(),
            serde_json::json!(extracted_at.to_rfc3339()),
        );
        self.metadata.insert(
            "parser_version".to_string(),
            serde_json::json!(parser_version),
        );
        self.metadata.insert(
            "transformation_log".to_string(),
            serde_json::json!(transformation_log),
        );
    }

    /// Set confidence score and reasons
    pub fn set_confidence(&mut self, score: f64, reasons: Vec<String>) {
        self.metadata
            .insert("confidence_score".to_string(), serde_json::json!(score));
        self.metadata.insert(
            "confidence_reasons".to_string(),
            serde_json::json!(reasons),
        );
    }

    /// Set verification status
    pub fn set_verification(&mut self, verified: bool, verifier: &str, verified_at: DateTime<Utc>) {
        self.metadata
            .insert("verified".to_string(), serde_json::json!(verified));
        self.metadata
            .insert("verified_by".to_string(), serde_json::json!(verifier));
        self.metadata.insert(
            "verified_at".to_string(),
            serde_json::json!(verified_at.to_rfc3339()),
        );
    }

    /// Get metadata value by key
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Check if metadata key exists
    pub fn has_metadata(&self, key: &str) -> bool {
        self.metadata.contains_key(key)
    }
}

/// Event for audit trail (Rich Hickey: "Every change is an event")
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub entity_type: String,
    pub entity_id: String,
    pub data: serde_json::Value,
    pub actor: String,
}

impl Event {
    pub fn new(
        event_type: &str,
        entity_type: &str,
        entity_id: &str,
        data: serde_json::Value,
        actor: &str,
    ) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: event_type.to_string(),
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            data,
            actor: actor.to_string(),
        }
    }
}

pub fn setup_database(conn: &Connection) -> Result<()> {
    // Enable WAL mode for crash recovery
    conn.pragma_update(None, "journal_mode", "WAL")?;

    // ==========================================================================
    // Transactions Table (with extensible metadata column)
    // Badge 19: Added temporal fields (tx_uuid, version, time model)
    // ==========================================================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            idempotency_hash TEXT UNIQUE NOT NULL,
            date TEXT NOT NULL,
            description TEXT NOT NULL,
            amount_original TEXT NOT NULL,
            amount_numeric REAL NOT NULL,
            transaction_type TEXT NOT NULL,
            category TEXT NOT NULL,
            merchant TEXT NOT NULL,
            currency TEXT NOT NULL,
            account_name TEXT NOT NULL,
            account_number TEXT NOT NULL,
            bank TEXT NOT NULL,
            source_file TEXT NOT NULL,
            line_number TEXT NOT NULL,
            classification_notes TEXT,
            metadata TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            -- Badge 19: Time & Identity Model (Rich Hickey's philosophy)
            tx_uuid TEXT UNIQUE,
            version INTEGER DEFAULT 1,
            system_time TEXT,
            valid_from TEXT,
            valid_until TEXT,
            previous_version_id TEXT
        )",
        [],
    )?;

    // ==========================================================================
    // Events Table (audit trail / event sourcing)
    // ==========================================================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            event_id TEXT UNIQUE NOT NULL,
            timestamp TEXT NOT NULL,
            event_type TEXT NOT NULL,
            entity_type TEXT NOT NULL,
            entity_id TEXT NOT NULL,
            data TEXT NOT NULL,
            actor TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // ==========================================================================
    // Indexes
    // ==========================================================================
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_idempotency_hash ON transactions(idempotency_hash)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_date ON transactions(date)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_bank ON transactions(bank)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_events_entity ON events(entity_type, entity_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp)",
        [],
    )?;

    Ok(())
}

pub fn load_csv(csv_path: &Path) -> Result<Vec<Transaction>> {
    let mut rdr = csv::Reader::from_path(csv_path).context("Failed to open CSV file")?;

    let mut transactions = Vec::new();

    for result in rdr.deserialize() {
        let mut transaction: Transaction = result.context("Failed to deserialize transaction")?;

        // Initialize temporal fields (UUID, version, timestamps) - Badge 19
        transaction.init_temporal_fields();

        // Add provenance metadata
        transaction.set_provenance(
            Utc::now(),
            "csv_loader_v1.0",
            vec!["loaded_from_csv".to_string()],
        );

        transactions.push(transaction);
    }

    Ok(transactions)
}

pub fn insert_transactions(conn: &Connection, transactions: &[Transaction]) -> Result<usize> {
    let mut inserted = 0;
    let mut duplicates = 0;

    for tx in transactions {
        let hash = tx.compute_idempotency_hash();

        // Serialize metadata to JSON
        let metadata_json = serde_json::to_string(&tx.metadata)?;

        // Serialize temporal fields (Badge 19)
        let system_time_str = tx.system_time.map(|dt| dt.to_rfc3339());
        let valid_from_str = tx.valid_from.map(|dt| dt.to_rfc3339());
        let valid_until_str = tx.valid_until.map(|dt| dt.to_rfc3339());

        let result = conn.execute(
            "INSERT INTO transactions (
                idempotency_hash, date, description, amount_original, amount_numeric,
                transaction_type, category, merchant, currency, account_name,
                account_number, bank, source_file, line_number, classification_notes,
                metadata,
                tx_uuid, version, system_time, valid_from, valid_until, previous_version_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)",
            params![
                hash,
                tx.date,
                tx.description,
                tx.amount_original,
                tx.amount_numeric,
                tx.transaction_type,
                tx.category,
                tx.merchant,
                tx.currency,
                tx.account_name,
                tx.account_number,
                tx.bank,
                tx.source_file,
                tx.line_number,
                tx.classification_notes,
                metadata_json,
                // Badge 19 temporal fields
                if tx.id.is_empty() { None } else { Some(&tx.id) },
                tx.version,
                system_time_str,
                valid_from_str,
                valid_until_str,
                tx.previous_version_id,
            ],
        );

        match result {
            Ok(_) => {
                inserted += 1;

                // Log event to audit trail
                let event = Event::new(
                    "transaction_added",
                    "transaction",
                    &hash,
                    serde_json::json!({
                        "bank": tx.bank,
                        "amount": tx.amount_numeric,
                        "source_file": tx.source_file,
                    }),
                    "csv_importer",
                );
                let _ = insert_event(conn, &event);
            }
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                duplicates += 1;
            }
            Err(e) => return Err(e.into()),
        }
    }

    println!("✓ Inserted: {} transactions", inserted);
    println!("✓ Skipped duplicates: {}", duplicates);

    Ok(inserted)
}

/// Insert event into audit trail
pub fn insert_event(conn: &Connection, event: &Event) -> Result<()> {
    let data_json = serde_json::to_string(&event.data)?;

    conn.execute(
        "INSERT INTO events (
            event_id, timestamp, event_type, entity_type, entity_id, data, actor
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            event.event_id,
            event.timestamp.to_rfc3339(),
            event.event_type,
            event.entity_type,
            event.entity_id,
            data_json,
            event.actor,
        ],
    )?;

    Ok(())
}

/// Get events for a specific entity
pub fn get_events_for_entity(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
) -> Result<Vec<Event>> {
    let mut stmt = conn.prepare(
        "SELECT event_id, timestamp, event_type, entity_type, entity_id, data, actor
         FROM events
         WHERE entity_type = ?1 AND entity_id = ?2
         ORDER BY timestamp DESC",
    )?;

    let events = stmt
        .query_map(params![entity_type, entity_id], |row| {
            let timestamp_str: String = row.get(1)?;
            let data_json: String = row.get(5)?;

            Ok(Event {
                event_id: row.get(0)?,
                timestamp: DateTime::parse_from_rfc3339(&timestamp_str)
                    .map_err(|e| rusqlite::Error::InvalidQuery)?
                    .with_timezone(&Utc),
                event_type: row.get(2)?,
                entity_type: row.get(3)?,
                entity_id: row.get(4)?,
                data: serde_json::from_str(&data_json)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?,
                actor: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(events)
}

pub fn get_all_transactions(conn: &Connection) -> Result<Vec<Transaction>> {
    let mut stmt = conn.prepare(
        "SELECT date, description, amount_original, amount_numeric,
                transaction_type, category, merchant, currency,
                account_name, account_number, bank, source_file,
                line_number, classification_notes, metadata,
                tx_uuid, version, system_time, valid_from, valid_until, previous_version_id
         FROM transactions
         ORDER BY date DESC",
    )?;

    let transactions = stmt
        .query_map([], |row| {
            let metadata_json: Option<String> = row.get(14)?;
            let metadata = if let Some(json_str) = metadata_json {
                serde_json::from_str(&json_str).unwrap_or_default()
            } else {
                HashMap::new()
            };

            // Parse temporal fields (Badge 19)
            let tx_uuid: Option<String> = row.get(15)?;
            let version: Option<i64> = row.get(16)?;
            let system_time_str: Option<String> = row.get(17)?;
            let valid_from_str: Option<String> = row.get(18)?;
            let valid_until_str: Option<String> = row.get(19)?;
            let previous_version_id: Option<String> = row.get(20)?;

            let system_time = system_time_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc));
            let valid_from = valid_from_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc));
            let valid_until = valid_until_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc));

            Ok(Transaction {
                date: row.get(0)?,
                description: row.get(1)?,
                amount_original: row.get(2)?,
                amount_numeric: row.get(3)?,
                transaction_type: row.get(4)?,
                category: row.get(5)?,
                merchant: row.get(6)?,
                currency: row.get(7)?,
                account_name: row.get(8)?,
                account_number: row.get(9)?,
                bank: row.get(10)?,
                source_file: row.get(11)?,
                line_number: row.get(12)?,
                classification_notes: row.get(13)?,
                // Badge 19 fields
                id: tx_uuid.unwrap_or_default(),
                version: version.unwrap_or(0),
                system_time,
                valid_from,
                valid_until,
                previous_version_id,
                metadata,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(transactions)
}

pub fn verify_count(conn: &Connection) -> Result<i64> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM transactions", [], |row| row.get(0))?;

    Ok(count)
}

/// Migrate existing transactions to have UUIDs (Badge 19)
/// Call this ONCE after upgrading to Badge 19 if you have existing data
pub fn migrate_add_uuids(conn: &Connection) -> Result<usize> {
    let now = Utc::now();
    let now_str = now.to_rfc3339();

    // Find transactions without UUIDs
    let mut stmt = conn.prepare(
        "SELECT id FROM transactions WHERE tx_uuid IS NULL OR tx_uuid = ''"
    )?;

    let row_ids: Vec<i64> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    let mut updated = 0;

    // Update each transaction with UUID and temporal fields
    for row_id in row_ids {
        let uuid = uuid::Uuid::new_v4().to_string();

        conn.execute(
            "UPDATE transactions
             SET tx_uuid = ?1,
                 version = COALESCE(version, 1),
                 system_time = COALESCE(system_time, ?2),
                 valid_from = COALESCE(valid_from, ?2)
             WHERE id = ?3",
            params![uuid, now_str, row_id],
        )?;

        updated += 1;
    }

    println!("✅ Migration complete: Added UUIDs to {} transactions", updated);
    Ok(updated)
}

/// Source file statistics
#[derive(Debug, Clone)]
pub struct SourceFileStat {
    pub source_file: String,
    pub bank: String,
    pub transaction_count: i64,
    pub total_expenses: f64,
    pub total_income: f64,
    pub date_range: String,
}

/// Get statistics grouped by source file
pub fn get_source_file_stats(conn: &Connection) -> Result<Vec<SourceFileStat>> {
    let mut stmt = conn.prepare(
        "SELECT
            source_file,
            bank,
            COUNT(*) as count,
            SUM(CASE WHEN transaction_type = 'GASTO' THEN ABS(amount_numeric) ELSE 0 END) as expenses,
            SUM(CASE WHEN transaction_type = 'INGRESO' THEN ABS(amount_numeric) ELSE 0 END) as income,
            MIN(date) || ' - ' || MAX(date) as date_range
         FROM transactions
         GROUP BY source_file, bank
         ORDER BY bank, source_file",
    )?;

    let stats = stmt
        .query_map([], |row| {
            Ok(SourceFileStat {
                source_file: row.get(0)?,
                bank: row.get(1)?,
                transaction_count: row.get(2)?,
                total_expenses: row.get(3)?,
                total_income: row.get(4)?,
                date_range: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(stats)
}

/// Get transactions by source file
pub fn get_transactions_by_source(
    conn: &Connection,
    source_file: &str,
) -> Result<Vec<Transaction>> {
    let mut stmt = conn.prepare(
        "SELECT date, description, amount_original, amount_numeric,
                transaction_type, category, merchant, currency,
                account_name, account_number, bank, source_file,
                line_number, classification_notes, metadata,
                tx_uuid, version, system_time, valid_from, valid_until, previous_version_id
         FROM transactions
         WHERE source_file = ?1
         ORDER BY date DESC",
    )?;

    let transactions = stmt
        .query_map([source_file], |row| {
            let metadata_json: Option<String> = row.get(14)?;
            let metadata = if let Some(json_str) = metadata_json {
                serde_json::from_str(&json_str).unwrap_or_default()
            } else {
                HashMap::new()
            };

            // Parse temporal fields (Badge 19)
            let tx_uuid: Option<String> = row.get(15)?;
            let version: Option<i64> = row.get(16)?;
            let system_time_str: Option<String> = row.get(17)?;
            let valid_from_str: Option<String> = row.get(18)?;
            let valid_until_str: Option<String> = row.get(19)?;
            let previous_version_id: Option<String> = row.get(20)?;

            let system_time = system_time_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc));
            let valid_from = valid_from_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc));
            let valid_until = valid_until_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc));

            Ok(Transaction {
                date: row.get(0)?,
                description: row.get(1)?,
                amount_original: row.get(2)?,
                amount_numeric: row.get(3)?,
                transaction_type: row.get(4)?,
                category: row.get(5)?,
                merchant: row.get(6)?,
                currency: row.get(7)?,
                account_name: row.get(8)?,
                account_number: row.get(9)?,
                bank: row.get(10)?,
                source_file: row.get(11)?,
                line_number: row.get(12)?,
                classification_notes: row.get(13)?,
                // Badge 19 fields
                id: tx_uuid.unwrap_or_default(),
                version: version.unwrap_or(0),
                system_time,
                valid_from,
                valid_until,
                previous_version_id,
                metadata,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(transactions)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create test transactions with all required fields
    fn create_test_transaction(
        date: &str,
        description: &str,
        amount: f64,
        tx_type: &str,
        category: &str,
        merchant: &str,
    ) -> Transaction {
        Transaction {
            date: date.to_string(),
            description: description.to_string(),
            amount_original: format!("${:.2}", amount.abs()),
            amount_numeric: amount,
            transaction_type: tx_type.to_string(),
            category: category.to_string(),
            merchant: merchant.to_string(),
            currency: "USD".to_string(),
            account_name: "Test Account".to_string(),
            account_number: "1234".to_string(),
            bank: "Test Bank".to_string(),
            source_file: "test.csv".to_string(),
            line_number: "1".to_string(),
            classification_notes: "".to_string(),
            // Badge 19 fields
            id: String::new(),  // Will be set by init_temporal_fields()
            version: 0,
            system_time: None,
            valid_from: None,
            valid_until: None,
            previous_version_id: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_idempotency_import_twice() {
        // Create temporary database
        let conn = Connection::open_in_memory().unwrap();
        setup_database(&conn).unwrap();

        // Create test transactions using helper
        let transactions = vec![
            create_test_transaction(
                "12/31/2024",
                "STARBUCKS #12345",
                -45.99,
                "GASTO",
                "Dining",
                "STARBUCKS",
            ),
            create_test_transaction(
                "12/30/2024",
                "AMAZON PURCHASE",
                -120.50,
                "GASTO",
                "Shopping",
                "AMAZON",
            ),
            create_test_transaction(
                "12/29/2024",
                "SALARY DEPOSIT",
                2000.00,
                "INGRESO",
                "Income",
                "EMPLOYER",
            ),
        ];

        println!("Created {} test transactions", transactions.len());

        // First import
        let inserted1 = insert_transactions(&conn, &transactions).unwrap();
        let count1 = verify_count(&conn).unwrap();

        println!(
            "First import: {} inserted, {} total in DB",
            inserted1, count1
        );

        // Second import (same transactions)
        let inserted2 = insert_transactions(&conn, &transactions).unwrap();
        let count2 = verify_count(&conn).unwrap();

        println!(
            "Second import: {} inserted, {} total in DB",
            inserted2, count2
        );

        // Assertions
        assert_eq!(inserted1, 3, "First import should insert 3 transactions");
        assert_eq!(
            count1, 3,
            "Database should have 3 transactions after first import"
        );
        assert_eq!(
            inserted2, 0,
            "Second import should insert 0 transactions (all duplicates)"
        );
        assert_eq!(
            count2, 3,
            "Database should still have 3 transactions after second import"
        );

        println!("✅ Idempotency test PASSED: 0 duplicates inserted on second import");
    }

    #[test]
    fn test_compute_idempotency_hash() {
        let tx = create_test_transaction(
            "12/31/2024",
            "TEST PURCHASE",
            -50.00,
            "GASTO",
            "Test",
            "TEST MERCHANT",
        );

        let hash1 = tx.compute_idempotency_hash();
        let hash2 = tx.compute_idempotency_hash();

        println!("Hash: {}", hash1);

        // Same transaction should produce same hash
        assert_eq!(hash1, hash2, "Same transaction should produce same hash");
        assert_eq!(
            hash1.len(),
            64,
            "SHA-256 hash should be 64 hex characters"
        );

        println!("✅ Idempotency hash test PASSED");
    }

    #[test]
    fn test_extensible_metadata() {
        let mut tx = create_test_transaction(
            "12/31/2024",
            "TEST",
            -50.00,
            "GASTO",
            "Test",
            "TEST",
        );

        // Add provenance
        tx.set_provenance(
            Utc::now(),
            "test_parser_v1.0",
            vec!["step1".to_string(), "step2".to_string()],
        );

        // Add confidence
        tx.set_confidence(0.95, vec!["rule_match".to_string()]);

        // Verify metadata
        assert!(tx.has_metadata("extracted_at"));
        assert!(tx.has_metadata("parser_version"));
        assert!(tx.has_metadata("confidence_score"));

        println!("✅ Extensible metadata test PASSED");
    }

    #[test]
    fn test_event_log() {
        let conn = Connection::open_in_memory().unwrap();
        setup_database(&conn).unwrap();

        let event = Event::new(
            "test_event",
            "transaction",
            "test_id_123",
            serde_json::json!({"test": "data"}),
            "test_actor",
        );

        insert_event(&conn, &event).unwrap();

        let events = get_events_for_entity(&conn, "transaction", "test_id_123").unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "test_event");
        assert_eq!(events[0].actor, "test_actor");

        println!("✅ Event log test PASSED");
    }
}
