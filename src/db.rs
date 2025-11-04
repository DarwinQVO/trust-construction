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
    // EXTENSIBLE METADATA (can grow without schema changes)
    // Following Rich Hickey's philosophy: "Aggregates as maps, not structs"
    // ========================================================================
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Transaction {
    /// Compute idempotency hash for duplicate detection
    pub fn compute_idempotency_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!(
            "{}{}{}{}",
            self.date, self.amount_numeric, self.merchant, self.bank
        ));
        format!("{:x}", hasher.finalize())
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
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
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

        let result = conn.execute(
            "INSERT INTO transactions (
                idempotency_hash, date, description, amount_original, amount_numeric,
                transaction_type, category, merchant, currency, account_name,
                account_number, bank, source_file, line_number, classification_notes,
                metadata
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
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
                line_number, classification_notes, metadata
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
                line_number, classification_notes, metadata
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
                metadata,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(transactions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idempotency_import_twice() {
        // Create temporary database
        let conn = Connection::open_in_memory().unwrap();
        setup_database(&conn).unwrap();

        // Create test transactions
        let transactions = vec![
            Transaction {
                date: "12/31/2024".to_string(),
                description: "STARBUCKS #12345".to_string(),
                amount_original: "-45.99".to_string(),
                amount_numeric: -45.99,
                transaction_type: "GASTO".to_string(),
                category: "Dining".to_string(),
                merchant: "STARBUCKS".to_string(),
                currency: "USD".to_string(),
                account_name: "Checking".to_string(),
                account_number: "1234".to_string(),
                bank: "BofA".to_string(),
                source_file: "test_idempotency.csv".to_string(),
                line_number: "2".to_string(),
                classification_notes: "".to_string(),
                metadata: HashMap::new(),
            },
            Transaction {
                date: "12/30/2024".to_string(),
                description: "AMAZON PURCHASE".to_string(),
                amount_original: "-120.50".to_string(),
                amount_numeric: -120.50,
                transaction_type: "GASTO".to_string(),
                category: "Shopping".to_string(),
                merchant: "AMAZON".to_string(),
                currency: "USD".to_string(),
                account_name: "Checking".to_string(),
                account_number: "1234".to_string(),
                bank: "BofA".to_string(),
                source_file: "test_idempotency.csv".to_string(),
                line_number: "3".to_string(),
                classification_notes: "".to_string(),
                metadata: HashMap::new(),
            },
            Transaction {
                date: "12/29/2024".to_string(),
                description: "SALARY DEPOSIT".to_string(),
                amount_original: "2000.00".to_string(),
                amount_numeric: 2000.00,
                transaction_type: "INGRESO".to_string(),
                category: "Income".to_string(),
                merchant: "EMPLOYER".to_string(),
                currency: "USD".to_string(),
                account_name: "Checking".to_string(),
                account_number: "1234".to_string(),
                bank: "BofA".to_string(),
                source_file: "test_idempotency.csv".to_string(),
                line_number: "4".to_string(),
                classification_notes: "".to_string(),
                metadata: HashMap::new(),
            },
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
        let tx = Transaction {
            date: "12/31/2024".to_string(),
            description: "TEST PURCHASE".to_string(),
            amount_original: "-50.00".to_string(),
            amount_numeric: -50.00,
            transaction_type: "GASTO".to_string(),
            category: "Test".to_string(),
            merchant: "TEST MERCHANT".to_string(),
            currency: "USD".to_string(),
            account_name: "Test Account".to_string(),
            account_number: "1234".to_string(),
            bank: "BofA".to_string(),
            source_file: "test.csv".to_string(),
            line_number: "1".to_string(),
            classification_notes: "".to_string(),
            metadata: HashMap::new(),
        };

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
        let mut tx = Transaction {
            date: "12/31/2024".to_string(),
            description: "TEST".to_string(),
            amount_original: "-50.00".to_string(),
            amount_numeric: -50.00,
            transaction_type: "GASTO".to_string(),
            category: "Test".to_string(),
            merchant: "TEST".to_string(),
            currency: "USD".to_string(),
            account_name: "Test".to_string(),
            account_number: "1234".to_string(),
            bank: "BofA".to_string(),
            source_file: "test.csv".to_string(),
            line_number: "1".to_string(),
            classification_notes: "".to_string(),
            metadata: HashMap::new(),
        };

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
