// ⚖️ Reconciliation Engine - Validate balances match
// Badge 19B: Ensures opening_balance + credits - debits = closing_balance
//
// Following the formula:
//   opening_balance + total_credits - total_debits = closing_balance
//
// This is CRITICAL for Trust Construction - without reconciliation,
// you cannot validate that your transaction sums are correct.

use crate::db::Transaction;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

// ============================================================================
// RECONCILIATION RESULT
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReconciliationResult {
    /// All balances match perfectly
    Balanced {
        opening_balance: f64,
        total_credits: f64,
        total_debits: f64,
        closing_balance: f64,
    },

    /// Balances don't match - off by small amount (< $10)
    MinorDiscrepancy {
        expected_balance: f64,
        actual_balance: f64,
        difference: f64,
        tolerance: f64,
    },

    /// Balances don't match - significant difference (>= $10)
    MajorDiscrepancy {
        expected_balance: f64,
        actual_balance: f64,
        difference: f64,
        missing_transactions: Vec<String>,
    },
}

impl ReconciliationResult {
    pub fn is_balanced(&self) -> bool {
        matches!(self, ReconciliationResult::Balanced { .. })
    }

    pub fn has_discrepancy(&self) -> bool {
        !self.is_balanced()
    }

    pub fn difference(&self) -> f64 {
        match self {
            ReconciliationResult::Balanced { .. } => 0.0,
            ReconciliationResult::MinorDiscrepancy { difference, .. } => *difference,
            ReconciliationResult::MajorDiscrepancy { difference, .. } => *difference,
        }
    }
}

// ============================================================================
// STATEMENT METADATA (from bank statements)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementMetadata {
    pub account_name: String,
    pub statement_period: String,
    pub opening_balance: f64,
    pub closing_balance: f64,
    pub statement_date: NaiveDate,
}

// ============================================================================
// RECONCILIATION REPORT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationReport {
    pub statement: StatementMetadata,
    pub result: ReconciliationResult,
    pub transaction_count: usize,
    pub total_credits: f64,
    pub total_debits: f64,
    pub calculated_balance: f64,
    pub discrepancies: Vec<Discrepancy>,
    pub reconciled_at: chrono::DateTime<chrono::Utc>,
}

impl ReconciliationReport {
    pub fn is_balanced(&self) -> bool {
        self.result.is_balanced()
    }

    pub fn summary(&self) -> String {
        format!(
            "Reconciliation for {} ({}): {} transactions, calculated ${:.2}, expected ${:.2}, difference ${:.2}",
            self.statement.account_name,
            self.statement.statement_period,
            self.transaction_count,
            self.calculated_balance,
            self.statement.closing_balance,
            self.result.difference()
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discrepancy {
    pub description: String,
    pub amount: f64,
    pub category: DiscrepancyCategory,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiscrepancyCategory {
    MissingTransaction,
    DuplicateTransaction,
    AmountMismatch,
    DateMismatch,
}

// ============================================================================
// RECONCILIATION ENGINE
// ============================================================================

pub struct ReconciliationEngine {
    /// Tolerance for floating-point comparisons (default: $0.01)
    pub tolerance: f64,

    /// Threshold for minor vs major discrepancy (default: $10.00)
    pub major_discrepancy_threshold: f64,
}

impl ReconciliationEngine {
    pub fn new() -> Self {
        ReconciliationEngine {
            tolerance: 0.01,
            major_discrepancy_threshold: 10.0,
        }
    }

    pub fn with_tolerance(tolerance: f64) -> Self {
        ReconciliationEngine {
            tolerance,
            major_discrepancy_threshold: 10.0,
        }
    }

    pub fn with_thresholds(tolerance: f64, major_threshold: f64) -> Self {
        ReconciliationEngine {
            tolerance,
            major_discrepancy_threshold: major_threshold,
        }
    }

    /// Reconcile transactions against statement metadata
    ///
    /// Formula: opening_balance + credits - debits = closing_balance
    ///
    /// Example:
    /// ```
    /// use trust_construction::{ReconciliationEngine, StatementMetadata, Transaction};
    /// use chrono::NaiveDate;
    ///
    /// let engine = ReconciliationEngine::new();
    /// let statement = StatementMetadata {
    ///     account_name: "BofA Checking".to_string(),
    ///     statement_period: "January 2025".to_string(),
    ///     opening_balance: 1000.0,
    ///     closing_balance: 2200.0,
    ///     statement_date: NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
    /// };
    ///
    /// let report = engine.reconcile(&transactions, &statement);
    /// assert!(report.is_balanced());
    /// ```
    pub fn reconcile(
        &self,
        transactions: &[Transaction],
        statement: &StatementMetadata,
    ) -> ReconciliationReport {
        let total_credits = self.calculate_credits(transactions);
        let total_debits = self.calculate_debits(transactions);

        // Formula: opening + credits - debits = closing
        let calculated_balance = statement.opening_balance + total_credits - total_debits;

        let difference = (calculated_balance - statement.closing_balance).abs();

        let result = if difference < self.tolerance {
            ReconciliationResult::Balanced {
                opening_balance: statement.opening_balance,
                total_credits,
                total_debits,
                closing_balance: statement.closing_balance,
            }
        } else if difference < self.major_discrepancy_threshold {
            // Minor discrepancy (< $10 by default)
            ReconciliationResult::MinorDiscrepancy {
                expected_balance: statement.closing_balance,
                actual_balance: calculated_balance,
                difference,
                tolerance: self.tolerance,
            }
        } else {
            // Major discrepancy (>= $10 by default)
            ReconciliationResult::MajorDiscrepancy {
                expected_balance: statement.closing_balance,
                actual_balance: calculated_balance,
                difference,
                missing_transactions: vec![], // TODO: detect missing transactions
            }
        };

        let discrepancies = self.detect_discrepancies(transactions, statement, difference);

        ReconciliationReport {
            statement: statement.clone(),
            result,
            transaction_count: transactions.len(),
            total_credits,
            total_debits,
            calculated_balance,
            discrepancies,
            reconciled_at: chrono::Utc::now(),
        }
    }

    /// Calculate total credits (INGRESO transactions)
    ///
    /// Credits are positive transactions that increase your balance:
    /// - Salary deposits
    /// - Income from Stripe
    /// - Refunds
    fn calculate_credits(&self, transactions: &[Transaction]) -> f64 {
        transactions
            .iter()
            .filter(|tx| tx.transaction_type == "INGRESO")
            .map(|tx| tx.amount_numeric.abs())
            .sum()
    }

    /// Calculate total debits (GASTO + PAGO_TARJETA transactions)
    ///
    /// Debits are negative transactions that decrease your balance:
    /// - Purchases (GASTO)
    /// - Credit card payments (PAGO_TARJETA)
    fn calculate_debits(&self, transactions: &[Transaction]) -> f64 {
        transactions
            .iter()
            .filter(|tx| {
                tx.transaction_type == "GASTO" || tx.transaction_type == "PAGO_TARJETA"
            })
            .map(|tx| tx.amount_numeric.abs())
            .sum()
    }

    /// Detect specific discrepancies
    ///
    /// Future improvements:
    /// - Detect missing transactions (compare with statement line items)
    /// - Detect duplicate transactions (using DeduplicationEngine)
    /// - Detect date mismatches
    fn detect_discrepancies(
        &self,
        _transactions: &[Transaction],
        _statement: &StatementMetadata,
        difference: f64,
    ) -> Vec<Discrepancy> {
        let mut discrepancies = Vec::new();

        if difference > self.tolerance {
            discrepancies.push(Discrepancy {
                description: format!("Balance mismatch: ${:.2} difference", difference),
                amount: difference,
                category: DiscrepancyCategory::AmountMismatch,
            });
        }

        // TODO: Detect missing transactions
        // Compare transaction list with statement line items

        // TODO: Detect duplicate transactions
        // Use DeduplicationEngine to find potential duplicates

        // TODO: Detect date mismatches
        // Check if transaction dates fall within statement period

        discrepancies
    }

    /// Quick check if transactions balance to expected amount
    pub fn quick_balance_check(
        &self,
        transactions: &[Transaction],
        expected_balance: f64,
        opening_balance: f64,
    ) -> bool {
        let credits = self.calculate_credits(transactions);
        let debits = self.calculate_debits(transactions);
        let calculated = opening_balance + credits - debits;

        (calculated - expected_balance).abs() < self.tolerance
    }
}

impl Default for ReconciliationEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_transaction(date: &str, amount: f64, tx_type: &str) -> Transaction {
        Transaction {
            date: date.to_string(),
            description: format!("Test transaction: {}", tx_type),
            amount_original: format!("${:.2}", amount.abs()),
            amount_numeric: amount,
            transaction_type: tx_type.to_string(),
            category: "Test".to_string(),
            merchant: "Test Merchant".to_string(),
            currency: "USD".to_string(),
            account_name: "Test Account".to_string(),
            account_number: "1234".to_string(),
            bank: "Test Bank".to_string(),
            source_file: "test.csv".to_string(),
            line_number: "1".to_string(),
            classification_notes: "".to_string(),
            id: String::new(),
            version: 0,
            system_time: None,
            valid_from: None,
            valid_until: None,
            previous_version_id: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_reconciliation_balanced() {
        let engine = ReconciliationEngine::new();

        let transactions = vec![
            create_test_transaction("01/01/2025", 2000.0, "INGRESO"), // +2000 credit
            create_test_transaction("01/02/2025", -500.0, "GASTO"),   // -500 debit
            create_test_transaction("01/03/2025", -300.0, "GASTO"),   // -300 debit
        ];

        let statement = StatementMetadata {
            account_name: "Test Account".to_string(),
            statement_period: "January 2025".to_string(),
            opening_balance: 1000.0,
            closing_balance: 2200.0, // 1000 + 2000 - 500 - 300 = 2200 ✅
            statement_date: NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
        };

        let report = engine.reconcile(&transactions, &statement);

        assert_eq!(report.transaction_count, 3);
        assert_eq!(report.total_credits, 2000.0);
        assert_eq!(report.total_debits, 800.0);
        assert_eq!(report.calculated_balance, 2200.0);
        assert!(report.is_balanced());
        assert!(matches!(report.result, ReconciliationResult::Balanced { .. }));

        println!("✅ Test passed: {}", report.summary());
    }

    #[test]
    fn test_reconciliation_minor_discrepancy() {
        let engine = ReconciliationEngine::new();

        let transactions = vec![
            create_test_transaction("01/01/2025", 2000.0, "INGRESO"),
            create_test_transaction("01/02/2025", -500.0, "GASTO"),
        ];

        let statement = StatementMetadata {
            account_name: "Test Account".to_string(),
            statement_period: "January 2025".to_string(),
            opening_balance: 1000.0,
            closing_balance: 2495.0, // Off by $5 (should be 2500)
            statement_date: NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
        };

        let report = engine.reconcile(&transactions, &statement);

        assert!(!report.is_balanced());
        assert!(report.result.has_discrepancy());
        assert!(matches!(
            report.result,
            ReconciliationResult::MinorDiscrepancy { .. }
        ));

        if let ReconciliationResult::MinorDiscrepancy { difference, .. } = report.result {
            assert!((difference - 5.0).abs() < 0.01);
        }

        assert_eq!(report.discrepancies.len(), 1);
        assert_eq!(
            report.discrepancies[0].category,
            DiscrepancyCategory::AmountMismatch
        );

        println!("✅ Test passed: {}", report.summary());
    }

    #[test]
    fn test_reconciliation_major_discrepancy() {
        let engine = ReconciliationEngine::new();

        let transactions = vec![create_test_transaction("01/01/2025", 2000.0, "INGRESO")];

        let statement = StatementMetadata {
            account_name: "Test Account".to_string(),
            statement_period: "January 2025".to_string(),
            opening_balance: 1000.0,
            closing_balance: 3100.0, // Off by $100 (should be 3000)
            statement_date: NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
        };

        let report = engine.reconcile(&transactions, &statement);

        assert!(!report.is_balanced());
        assert!(matches!(
            report.result,
            ReconciliationResult::MajorDiscrepancy { .. }
        ));

        if let ReconciliationResult::MajorDiscrepancy { difference, .. } = report.result {
            assert!((difference - 100.0).abs() < 0.01);
        }

        println!("✅ Test passed: {}", report.summary());
    }

    #[test]
    fn test_calculate_credits() {
        let engine = ReconciliationEngine::new();

        let transactions = vec![
            create_test_transaction("01/01/2025", 2000.0, "INGRESO"),
            create_test_transaction("01/02/2025", 1500.0, "INGRESO"),
            create_test_transaction("01/03/2025", -500.0, "GASTO"), // Not a credit
        ];

        let credits = engine.calculate_credits(&transactions);
        assert_eq!(credits, 3500.0); // 2000 + 1500 = 3500

        println!("✅ Credits calculation test passed: ${:.2}", credits);
    }

    #[test]
    fn test_calculate_debits() {
        let engine = ReconciliationEngine::new();

        let transactions = vec![
            create_test_transaction("01/01/2025", -500.0, "GASTO"),
            create_test_transaction("01/02/2025", -300.0, "GASTO"),
            create_test_transaction("01/03/2025", -200.0, "PAGO_TARJETA"),
            create_test_transaction("01/04/2025", 2000.0, "INGRESO"), // Not a debit
        ];

        let debits = engine.calculate_debits(&transactions);
        assert_eq!(debits, 1000.0); // 500 + 300 + 200 = 1000

        println!("✅ Debits calculation test passed: ${:.2}", debits);
    }

    #[test]
    fn test_quick_balance_check() {
        let engine = ReconciliationEngine::new();

        let transactions = vec![
            create_test_transaction("01/01/2025", 2000.0, "INGRESO"),
            create_test_transaction("01/02/2025", -500.0, "GASTO"),
        ];

        // Correct balance
        assert!(engine.quick_balance_check(&transactions, 2500.0, 1000.0));

        // Incorrect balance
        assert!(!engine.quick_balance_check(&transactions, 2000.0, 1000.0));

        println!("✅ Quick balance check test passed");
    }

    #[test]
    fn test_reconciliation_result_methods() {
        let balanced = ReconciliationResult::Balanced {
            opening_balance: 1000.0,
            total_credits: 2000.0,
            total_debits: 500.0,
            closing_balance: 2500.0,
        };

        assert!(balanced.is_balanced());
        assert!(!balanced.has_discrepancy());
        assert_eq!(balanced.difference(), 0.0);

        let minor = ReconciliationResult::MinorDiscrepancy {
            expected_balance: 2500.0,
            actual_balance: 2495.0,
            difference: 5.0,
            tolerance: 0.01,
        };

        assert!(!minor.is_balanced());
        assert!(minor.has_discrepancy());
        assert_eq!(minor.difference(), 5.0);

        println!("✅ ReconciliationResult methods test passed");
    }
}
