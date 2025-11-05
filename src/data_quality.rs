// ✅ Data Quality Engine - Great Expectations style validation
// Badge 20: Validates all transaction fields + temporal integrity
//
// Inspired by Great Expectations (https://greatexpectations.io/)
// Provides comprehensive data quality checks with confidence scoring

use crate::db::Transaction;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

// ============================================================================
// VALIDATION RESULT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub rule_name: String,
    pub field: String,
    pub message: String,
    pub confidence: f64,
    pub severity: Severity,
}

impl ValidationResult {
    pub fn pass(rule_name: &str, field: &str, message: &str) -> Self {
        ValidationResult {
            passed: true,
            rule_name: rule_name.to_string(),
            field: field.to_string(),
            message: message.to_string(),
            confidence: 1.0,
            severity: Severity::Info,
        }
    }

    pub fn fail(rule_name: &str, field: &str, message: &str, severity: Severity) -> Self {
        ValidationResult {
            passed: false,
            rule_name: rule_name.to_string(),
            field: field.to_string(),
            message: message.to_string(),
            confidence: if severity == Severity::Critical {
                0.0
            } else {
                0.5
            },
            severity,
        }
    }
}

// ============================================================================
// QUALITY REPORT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    pub transaction_id: String,
    pub overall_quality: f64,
    pub overall_confidence: f64,
    pub validations: Vec<ValidationResult>,
    pub issues: Vec<QualityIssue>,
    pub passed_count: usize,
    pub failed_count: usize,
    pub needs_review: bool,
}

impl QualityReport {
    pub fn summary(&self) -> String {
        format!(
            "Quality: {:.1}%, Confidence: {:.1}%, Issues: {} ({} critical)",
            self.overall_quality * 100.0,
            self.overall_confidence * 100.0,
            self.issues.len(),
            self.issues
                .iter()
                .filter(|i| i.severity == Severity::Critical)
                .count()
        )
    }

    pub fn is_high_quality(&self) -> bool {
        self.overall_quality >= 0.8 && self.overall_confidence >= 0.7
    }

    pub fn has_critical_issues(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.severity == Severity::Critical)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    pub severity: Severity,
    pub field: String,
    pub issue: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Severity {
    Critical, // Data is invalid or missing critical information
    Warning,  // Data is questionable or incomplete
    Info,     // Data is valid but could be improved
}

// ============================================================================
// DATA QUALITY ENGINE
// ============================================================================

pub struct DataQualityEngine {
    /// Known valid categories
    known_categories: Vec<String>,

    /// Known valid banks
    known_banks: Vec<String>,

    /// Known valid transaction types
    known_types: Vec<String>,

    /// Minimum confidence threshold for "needs_review"
    review_threshold: f64,
}

impl DataQualityEngine {
    pub fn new() -> Self {
        DataQualityEngine {
            known_categories: vec![
                "Restaurants".to_string(),
                "Shopping".to_string(),
                "Transport".to_string(),
                "Entertainment".to_string(),
                "Groceries".to_string(),
                "Bills".to_string(),
                "Income".to_string(),
                "Payment".to_string(),
                "Transfer".to_string(),
                "Health".to_string(),
                "Education".to_string(),
                "Housing".to_string(),
                "Utilities".to_string(),
                "Insurance".to_string(),
                "Savings".to_string(),
                "Investment".to_string(),
                "Fees".to_string(),
                "Tax".to_string(),
                "Gift".to_string(),
                "Charity".to_string(),
                "Unknown".to_string(),
            ],
            known_banks: vec![
                "AppleCard".to_string(),
                "BofA".to_string(),
                "Bank of America".to_string(),
                "Stripe".to_string(),
                "Wise".to_string(),
                "Scotiabank".to_string(),
            ],
            known_types: vec![
                "GASTO".to_string(),
                "INGRESO".to_string(),
                "PAGO_TARJETA".to_string(),
                "TRASPASO".to_string(),
            ],
            review_threshold: 0.7,
        }
    }

    /// Validate a transaction and generate quality report
    pub fn validate(&self, tx: &Transaction) -> QualityReport {
        let mut validations = Vec::new();
        let mut issues = Vec::new();

        // Rule 1: Date format valid
        let date_result = self.validate_date(&tx.date);
        if !date_result.passed {
            issues.push(QualityIssue {
                severity: date_result.severity.clone(),
                field: "date".to_string(),
                issue: date_result.message.clone(),
                recommendation: "Fix date format to MM/DD/YYYY or YYYY-MM-DD".to_string(),
            });
        }
        validations.push(date_result);

        // Rule 2: Amount is numeric and non-zero
        let amount_result = self.validate_amount(tx.amount_numeric);
        if !amount_result.passed {
            issues.push(QualityIssue {
                severity: amount_result.severity.clone(),
                field: "amount".to_string(),
                issue: amount_result.message.clone(),
                recommendation: "Verify transaction amount is correct".to_string(),
            });
        }
        validations.push(amount_result);

        // Rule 3: Merchant not empty
        let merchant_result = self.validate_merchant(&tx.merchant);
        if !merchant_result.passed {
            issues.push(QualityIssue {
                severity: merchant_result.severity.clone(),
                field: "merchant".to_string(),
                issue: merchant_result.message.clone(),
                recommendation: "Add merchant information for better tracking".to_string(),
            });
        }
        validations.push(merchant_result);

        // Rule 4: Category is known
        let category_result = self.validate_category(&tx.category);
        if !category_result.passed {
            issues.push(QualityIssue {
                severity: category_result.severity.clone(),
                field: "category".to_string(),
                issue: category_result.message.clone(),
                recommendation: format!(
                    "Use one of known categories: {}",
                    self.known_categories.join(", ")
                ),
            });
        }
        validations.push(category_result);

        // Rule 5: Bank is known
        let bank_result = self.validate_bank(&tx.bank);
        if !bank_result.passed {
            issues.push(QualityIssue {
                severity: bank_result.severity.clone(),
                field: "bank".to_string(),
                issue: bank_result.message.clone(),
                recommendation: "Verify bank name matches known banks".to_string(),
            });
        }
        validations.push(bank_result);

        // Rule 6: Transaction type is valid
        let type_result = self.validate_transaction_type(&tx.transaction_type);
        if !type_result.passed {
            issues.push(QualityIssue {
                severity: type_result.severity.clone(),
                field: "transaction_type".to_string(),
                issue: type_result.message.clone(),
                recommendation: "Use GASTO, INGRESO, PAGO_TARJETA, or TRASPASO".to_string(),
            });
        }
        validations.push(type_result);

        // Rule 7: Description not empty
        let desc_result = self.validate_description(&tx.description);
        if !desc_result.passed {
            issues.push(QualityIssue {
                severity: desc_result.severity.clone(),
                field: "description".to_string(),
                issue: desc_result.message.clone(),
                recommendation: "Add transaction description for context".to_string(),
            });
        }
        validations.push(desc_result);

        // Rule 8: Currency is valid
        let currency_result = self.validate_currency(&tx.currency);
        if !currency_result.passed {
            issues.push(QualityIssue {
                severity: currency_result.severity.clone(),
                field: "currency".to_string(),
                issue: currency_result.message.clone(),
                recommendation: "Use ISO 4217 currency code (USD, EUR, etc.)".to_string(),
            });
        }
        validations.push(currency_result);

        // Rule 9: Account information present
        let account_result = self.validate_account(&tx.account_name, &tx.account_number);
        if !account_result.passed {
            issues.push(QualityIssue {
                severity: account_result.severity.clone(),
                field: "account".to_string(),
                issue: account_result.message.clone(),
                recommendation: "Add account name and number for proper tracking".to_string(),
            });
        }
        validations.push(account_result);

        // Rule 10: Provenance (source_file + line_number) present
        let provenance_result = self.validate_provenance(&tx.source_file, &tx.line_number);
        if !provenance_result.passed {
            issues.push(QualityIssue {
                severity: provenance_result.severity.clone(),
                field: "provenance".to_string(),
                issue: provenance_result.message.clone(),
                recommendation: "Add source_file and line_number for audit trail".to_string(),
            });
        }
        validations.push(provenance_result);

        // Rule 11: Temporal integrity (Badge 19 fields)
        if !tx.id.is_empty() {
            let temporal_result = self.validate_temporal_fields(tx);
            if !temporal_result.passed {
                issues.push(QualityIssue {
                    severity: temporal_result.severity.clone(),
                    field: "temporal".to_string(),
                    issue: temporal_result.message.clone(),
                    recommendation:
                        "Ensure UUID, version, and timestamps are properly initialized"
                            .to_string(),
                });
            }
            validations.push(temporal_result);
        }

        // Calculate overall metrics
        let passed_count = validations.iter().filter(|v| v.passed).count();
        let failed_count = validations.len() - passed_count;
        let overall_quality = passed_count as f64 / validations.len() as f64;

        // Calculate overall confidence (average of all confidences)
        let overall_confidence: f64 =
            validations.iter().map(|v| v.confidence).sum::<f64>() / validations.len() as f64;

        let needs_review = overall_confidence < self.review_threshold;

        QualityReport {
            transaction_id: tx.id.clone(),
            overall_quality,
            overall_confidence,
            validations,
            issues,
            passed_count,
            failed_count,
            needs_review,
        }
    }

    /// Batch validate multiple transactions
    pub fn validate_batch(&self, transactions: &[Transaction]) -> Vec<QualityReport> {
        transactions.iter().map(|tx| self.validate(tx)).collect()
    }

    /// Generate summary statistics for batch validation
    pub fn batch_summary(&self, reports: &[QualityReport]) -> BatchSummary {
        let total = reports.len();
        let high_quality = reports.iter().filter(|r| r.is_high_quality()).count();
        let needs_review = reports.iter().filter(|r| r.needs_review).count();
        let has_critical = reports.iter().filter(|r| r.has_critical_issues()).count();

        let avg_quality: f64 = reports.iter().map(|r| r.overall_quality).sum::<f64>() / total as f64;
        let avg_confidence: f64 =
            reports.iter().map(|r| r.overall_confidence).sum::<f64>() / total as f64;

        BatchSummary {
            total_transactions: total,
            high_quality_count: high_quality,
            needs_review_count: needs_review,
            critical_issues_count: has_critical,
            average_quality: avg_quality,
            average_confidence: avg_confidence,
        }
    }

    // ========================================================================
    // VALIDATION RULES
    // ========================================================================

    fn validate_date(&self, date: &str) -> ValidationResult {
        if date.is_empty() {
            return ValidationResult::fail(
                "date_not_empty",
                "date",
                "Date is empty",
                Severity::Critical,
            );
        }

        // Try parsing MM/DD/YYYY
        let parse_mdy = NaiveDate::parse_from_str(date, "%m/%d/%Y");
        if parse_mdy.is_ok() {
            return ValidationResult::pass("date_valid", "date", "Date format valid (MM/DD/YYYY)");
        }

        // Try parsing YYYY-MM-DD
        let parse_ymd = NaiveDate::parse_from_str(date, "%Y-%m-%d");
        if parse_ymd.is_ok() {
            return ValidationResult::pass("date_valid", "date", "Date format valid (YYYY-MM-DD)");
        }

        ValidationResult::fail(
            "date_invalid_format",
            "date",
            &format!("Invalid date format: {}", date),
            Severity::Critical,
        )
    }

    fn validate_amount(&self, amount: f64) -> ValidationResult {
        if amount == 0.0 {
            return ValidationResult::fail(
                "amount_zero",
                "amount",
                "Amount is zero",
                Severity::Warning,
            );
        }

        if amount.is_nan() || amount.is_infinite() {
            return ValidationResult::fail(
                "amount_invalid",
                "amount",
                "Amount is not a valid number",
                Severity::Critical,
            );
        }

        ValidationResult::pass(
            "amount_valid",
            "amount",
            &format!("Amount is valid: ${:.2}", amount.abs()),
        )
    }

    fn validate_merchant(&self, merchant: &str) -> ValidationResult {
        if merchant.is_empty() {
            return ValidationResult::fail(
                "merchant_empty",
                "merchant",
                "Merchant is empty",
                Severity::Warning,
            );
        }

        if merchant.len() < 2 {
            return ValidationResult::fail(
                "merchant_too_short",
                "merchant",
                "Merchant name too short",
                Severity::Warning,
            );
        }

        ValidationResult::pass(
            "merchant_present",
            "merchant",
            &format!("Merchant present: {}", merchant),
        )
    }

    fn validate_category(&self, category: &str) -> ValidationResult {
        if category.is_empty() {
            return ValidationResult::fail(
                "category_empty",
                "category",
                "Category is empty",
                Severity::Warning,
            );
        }

        if !self.known_categories.contains(&category.to_string()) {
            return ValidationResult::fail(
                "category_unknown",
                "category",
                &format!("Unknown category: {}", category),
                Severity::Info,
            );
        }

        ValidationResult::pass(
            "category_known",
            "category",
            &format!("Category is known: {}", category),
        )
    }

    fn validate_bank(&self, bank: &str) -> ValidationResult {
        if bank.is_empty() {
            return ValidationResult::fail(
                "bank_empty",
                "bank",
                "Bank is empty",
                Severity::Critical,
            );
        }

        let bank_lower = bank.to_lowercase();
        let matches = self
            .known_banks
            .iter()
            .any(|b| bank_lower.contains(&b.to_lowercase()));

        if !matches {
            return ValidationResult::fail(
                "bank_unknown",
                "bank",
                &format!("Unknown bank: {}", bank),
                Severity::Warning,
            );
        }

        ValidationResult::pass("bank_known", "bank", &format!("Bank is known: {}", bank))
    }

    fn validate_transaction_type(&self, tx_type: &str) -> ValidationResult {
        if tx_type.is_empty() {
            return ValidationResult::fail(
                "type_empty",
                "transaction_type",
                "Transaction type is empty",
                Severity::Critical,
            );
        }

        if !self.known_types.contains(&tx_type.to_string()) {
            return ValidationResult::fail(
                "type_unknown",
                "transaction_type",
                &format!("Unknown transaction type: {}", tx_type),
                Severity::Critical,
            );
        }

        ValidationResult::pass(
            "type_valid",
            "transaction_type",
            &format!("Transaction type valid: {}", tx_type),
        )
    }

    fn validate_description(&self, description: &str) -> ValidationResult {
        if description.is_empty() {
            return ValidationResult::fail(
                "description_empty",
                "description",
                "Description is empty",
                Severity::Info,
            );
        }

        if description.len() < 3 {
            return ValidationResult::fail(
                "description_too_short",
                "description",
                "Description too short",
                Severity::Info,
            );
        }

        ValidationResult::pass(
            "description_present",
            "description",
            "Description present",
        )
    }

    fn validate_currency(&self, currency: &str) -> ValidationResult {
        if currency.is_empty() {
            return ValidationResult::fail(
                "currency_empty",
                "currency",
                "Currency is empty",
                Severity::Warning,
            );
        }

        if currency.len() != 3 {
            return ValidationResult::fail(
                "currency_invalid_length",
                "currency",
                &format!("Currency code should be 3 characters: {}", currency),
                Severity::Warning,
            );
        }

        // Common currencies
        let common_currencies = vec!["USD", "EUR", "GBP", "CAD", "MXN", "JPY", "CNY"];
        if !common_currencies.contains(&currency) {
            return ValidationResult::fail(
                "currency_uncommon",
                "currency",
                &format!("Uncommon currency: {}", currency),
                Severity::Info,
            );
        }

        ValidationResult::pass(
            "currency_valid",
            "currency",
            &format!("Currency valid: {}", currency),
        )
    }

    fn validate_account(&self, account_name: &str, account_number: &str) -> ValidationResult {
        if account_name.is_empty() && account_number.is_empty() {
            return ValidationResult::fail(
                "account_missing",
                "account",
                "Both account name and number are empty",
                Severity::Warning,
            );
        }

        if account_name.is_empty() {
            return ValidationResult::fail(
                "account_name_empty",
                "account",
                "Account name is empty",
                Severity::Info,
            );
        }

        if account_number.is_empty() {
            return ValidationResult::fail(
                "account_number_empty",
                "account",
                "Account number is empty",
                Severity::Info,
            );
        }

        ValidationResult::pass(
            "account_complete",
            "account",
            "Account information complete",
        )
    }

    fn validate_provenance(&self, source_file: &str, line_number: &str) -> ValidationResult {
        if source_file.is_empty() {
            return ValidationResult::fail(
                "provenance_no_source",
                "provenance",
                "Source file is missing",
                Severity::Warning,
            );
        }

        if line_number.is_empty() {
            return ValidationResult::fail(
                "provenance_no_line",
                "provenance",
                "Line number is missing",
                Severity::Info,
            );
        }

        ValidationResult::pass(
            "provenance_complete",
            "provenance",
            &format!("Provenance: {}:{}", source_file, line_number),
        )
    }

    fn validate_temporal_fields(&self, tx: &Transaction) -> ValidationResult {
        let has_uuid = !tx.id.is_empty();
        let has_version = tx.version > 0;
        let has_system_time = tx.system_time.is_some();
        let has_valid_from = tx.valid_from.is_some();

        if !has_uuid {
            return ValidationResult::fail(
                "temporal_no_uuid",
                "temporal",
                "Missing UUID (Badge 19)",
                Severity::Critical,
            );
        }

        if !has_version {
            return ValidationResult::fail(
                "temporal_no_version",
                "temporal",
                "Missing version number (Badge 19)",
                Severity::Warning,
            );
        }

        if !has_system_time {
            return ValidationResult::fail(
                "temporal_no_system_time",
                "temporal",
                "Missing system_time (Badge 19)",
                Severity::Warning,
            );
        }

        if !has_valid_from {
            return ValidationResult::fail(
                "temporal_no_valid_from",
                "temporal",
                "Missing valid_from (Badge 19)",
                Severity::Warning,
            );
        }

        ValidationResult::pass(
            "temporal_complete",
            "temporal",
            "Temporal fields complete (Badge 19)",
        )
    }
}

impl Default for DataQualityEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// BATCH SUMMARY
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSummary {
    pub total_transactions: usize,
    pub high_quality_count: usize,
    pub needs_review_count: usize,
    pub critical_issues_count: usize,
    pub average_quality: f64,
    pub average_confidence: f64,
}

impl BatchSummary {
    pub fn summary(&self) -> String {
        format!(
            "{} transactions: {:.1}% quality, {:.1}% confidence | {} high quality, {} need review, {} critical",
            self.total_transactions,
            self.average_quality * 100.0,
            self.average_confidence * 100.0,
            self.high_quality_count,
            self.needs_review_count,
            self.critical_issues_count
        )
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_valid_transaction() -> Transaction {
        let mut tx = Transaction {
            date: "01/15/2025".to_string(),
            description: "Test purchase at Starbucks".to_string(),
            amount_original: "$45.99".to_string(),
            amount_numeric: -45.99,
            transaction_type: "GASTO".to_string(),
            category: "Restaurants".to_string(),
            merchant: "Starbucks".to_string(),
            currency: "USD".to_string(),
            account_name: "BofA Checking".to_string(),
            account_number: "*1234".to_string(),
            bank: "Bank of America".to_string(),
            source_file: "bofa_jan_2025.csv".to_string(),
            line_number: "23".to_string(),
            classification_notes: "".to_string(),
            id: "uuid-123".to_string(),
            version: 1,
            system_time: Some(chrono::Utc::now()),
            valid_from: Some(chrono::Utc::now()),
            valid_until: None,
            previous_version_id: None,
            metadata: HashMap::new(),
        };

        tx.init_temporal_fields();
        tx
    }

    #[test]
    fn test_validate_perfect_transaction() {
        let engine = DataQualityEngine::new();
        let tx = create_valid_transaction();

        let report = engine.validate(&tx);

        println!("Report: {}", report.summary());

        assert!(report.is_high_quality());
        assert!(!report.needs_review);
        assert!(!report.has_critical_issues());
        assert!(report.overall_quality >= 0.9);
        assert!(report.overall_confidence >= 0.9);
        assert_eq!(report.issues.len(), 0);
    }

    #[test]
    fn test_validate_missing_merchant() {
        let engine = DataQualityEngine::new();
        let mut tx = create_valid_transaction();
        tx.merchant = "".to_string();

        let report = engine.validate(&tx);

        // Merchant is Warning severity, not Critical, so overall quality can still be high
        // What matters is that the issue is detected
        assert!(report.issues.len() > 0);
        assert!(report.issues.iter().any(|i| i.field == "merchant"));
        assert!(report.issues.iter().any(|i| i.severity == Severity::Warning));
    }

    #[test]
    fn test_validate_invalid_date() {
        let engine = DataQualityEngine::new();
        let mut tx = create_valid_transaction();
        tx.date = "invalid-date".to_string();

        let report = engine.validate(&tx);

        assert!(report.has_critical_issues());
        assert!(report.issues.iter().any(|i| i.field == "date"));
    }

    #[test]
    fn test_validate_unknown_category() {
        let engine = DataQualityEngine::new();
        let mut tx = create_valid_transaction();
        tx.category = "RandomCategory".to_string();

        let report = engine.validate(&tx);

        println!("Report: {}", report.summary());

        // Unknown category is Info severity, not critical
        assert!(report.issues.len() > 0);
        assert!(report.issues.iter().any(|i| i.field == "category"));
    }

    #[test]
    fn test_validate_zero_amount() {
        let engine = DataQualityEngine::new();
        let mut tx = create_valid_transaction();
        tx.amount_numeric = 0.0;

        let report = engine.validate(&tx);

        assert!(report.issues.len() > 0);
        assert!(report.issues.iter().any(|i| i.field == "amount"));
    }

    #[test]
    fn test_validate_missing_temporal_fields() {
        let engine = DataQualityEngine::new();
        let mut tx = create_valid_transaction();
        tx.id = "".to_string();
        tx.version = 0;
        tx.system_time = None;

        let report = engine.validate(&tx);

        // Should not validate temporal fields if id is empty
        assert!(report.validations.iter().all(|v| v.field != "temporal"));
    }

    #[test]
    fn test_batch_validation() {
        let engine = DataQualityEngine::new();

        let transactions = vec![
            create_valid_transaction(),
            create_valid_transaction(),
            create_valid_transaction(),
        ];

        let reports = engine.validate_batch(&transactions);

        assert_eq!(reports.len(), 3);
        assert!(reports.iter().all(|r| r.is_high_quality()));

        let summary = engine.batch_summary(&reports);

        println!("Batch summary: {}", summary.summary());

        assert_eq!(summary.total_transactions, 3);
        assert_eq!(summary.high_quality_count, 3);
        assert_eq!(summary.needs_review_count, 0);
        assert_eq!(summary.critical_issues_count, 0);
    }

    #[test]
    fn test_quality_report_methods() {
        let engine = DataQualityEngine::new();
        let tx = create_valid_transaction();
        let report = engine.validate(&tx);

        // Test helper methods
        assert!(report.is_high_quality());
        assert!(!report.has_critical_issues());
        assert!(!report.needs_review);
        assert!(!report.summary().is_empty());
    }
}
