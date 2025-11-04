// 🔍 Deduplication Engine - Detect duplicate transactions
// Three strategies: Exact Match, Fuzzy Match, Transfer Pair

use crate::db::Transaction;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

// ============================================================================
// MATCH STRATEGY
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MatchStrategy {
    /// Exact match: same date, amount, merchant
    ExactMatch,

    /// Fuzzy match: similar date (±1 day), similar amount (±$0.50), similar merchant
    FuzzyMatch,

    /// Transfer pair: same date, opposite amounts, both TRASPASO
    TransferPair,
}

// ============================================================================
// DUPLICATE MATCH RESULT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateMatch {
    /// Index of first transaction
    pub tx1_index: usize,

    /// Index of second transaction
    pub tx2_index: usize,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,

    /// Which strategy detected this match
    pub strategy: MatchStrategy,

    /// Human-readable reason
    pub reason: String,
}

// ============================================================================
// DEDUPLICATION ENGINE
// ============================================================================

pub struct DeduplicationEngine {
    /// Confidence threshold for exact matches (default: 0.95)
    pub exact_match_threshold: f64,

    /// Confidence threshold for fuzzy matches (default: 0.70)
    pub fuzzy_match_threshold: f64,

    /// Confidence threshold for transfer pairs (default: 0.90)
    pub transfer_match_threshold: f64,

    /// Amount tolerance for fuzzy matching (default: $0.50)
    pub fuzzy_amount_tolerance: f64,

    /// Date tolerance for fuzzy matching in days (default: 1)
    pub fuzzy_date_tolerance_days: i64,
}

impl DeduplicationEngine {
    /// Create engine with default thresholds
    pub fn new() -> Self {
        DeduplicationEngine {
            exact_match_threshold: 0.95,
            fuzzy_match_threshold: 0.70,
            transfer_match_threshold: 0.90,
            fuzzy_amount_tolerance: 0.50,
            fuzzy_date_tolerance_days: 1,
        }
    }

    /// Find all duplicate matches in a list of transactions
    pub fn find_duplicates(&self, transactions: &[Transaction]) -> Vec<DuplicateMatch> {
        let mut matches = Vec::new();

        // Compare each transaction with every other transaction
        for i in 0..transactions.len() {
            for j in (i + 1)..transactions.len() {
                let tx1 = &transactions[i];
                let tx2 = &transactions[j];

                // Try exact match first (highest confidence)
                if let Some(m) = self.check_exact_match(i, j, tx1, tx2) {
                    matches.push(m);
                    continue;
                }

                // Try transfer pair detection
                if let Some(m) = self.check_transfer_pair(i, j, tx1, tx2) {
                    matches.push(m);
                    continue;
                }

                // Try fuzzy match (lowest confidence)
                if let Some(m) = self.check_fuzzy_match(i, j, tx1, tx2) {
                    matches.push(m);
                }
            }
        }

        matches
    }

    /// Strategy 1: Exact Match
    /// Same date, same amount, same merchant → 95%+ confidence
    fn check_exact_match(
        &self,
        i: usize,
        j: usize,
        tx1: &Transaction,
        tx2: &Transaction,
    ) -> Option<DuplicateMatch> {
        // Date must match exactly
        if tx1.date != tx2.date {
            return None;
        }

        // Amount must match exactly
        if (tx1.amount_numeric - tx2.amount_numeric).abs() > 0.001 {
            return None;
        }

        // Merchant must match exactly (case-insensitive)
        if tx1.merchant.to_lowercase() != tx2.merchant.to_lowercase() {
            return None;
        }

        Some(DuplicateMatch {
            tx1_index: i,
            tx2_index: j,
            confidence: self.exact_match_threshold,
            strategy: MatchStrategy::ExactMatch,
            reason: format!(
                "Exact match: {} | ${:.2} | {}",
                tx1.date, tx1.amount_numeric.abs(), tx1.merchant
            ),
        })
    }

    /// Strategy 2: Transfer Pair
    /// Same date, opposite amounts, both TRASPASO → 90%+ confidence
    fn check_transfer_pair(
        &self,
        i: usize,
        j: usize,
        tx1: &Transaction,
        tx2: &Transaction,
    ) -> Option<DuplicateMatch> {
        // Both must be TRASPASO
        if tx1.transaction_type != "TRASPASO" || tx2.transaction_type != "TRASPASO" {
            return None;
        }

        // Date must match exactly
        if tx1.date != tx2.date {
            return None;
        }

        // Amounts must be opposite (one positive, one negative, same magnitude)
        let sum = tx1.amount_numeric + tx2.amount_numeric;
        if sum.abs() > 0.01 {
            // Not opposite amounts
            return None;
        }

        Some(DuplicateMatch {
            tx1_index: i,
            tx2_index: j,
            confidence: self.transfer_match_threshold,
            strategy: MatchStrategy::TransferPair,
            reason: format!(
                "Transfer pair: {} | ${:.2} ↔ ${:.2}",
                tx1.date, tx1.amount_numeric, tx2.amount_numeric
            ),
        })
    }

    /// Strategy 3: Fuzzy Match
    /// Similar date (±1 day), similar amount (±$0.50), similar merchant → 70%+ confidence
    fn check_fuzzy_match(
        &self,
        i: usize,
        j: usize,
        tx1: &Transaction,
        tx2: &Transaction,
    ) -> Option<DuplicateMatch> {
        // Parse dates
        let date1 = match self.parse_date(&tx1.date) {
            Some(d) => d,
            None => return None,
        };
        let date2 = match self.parse_date(&tx2.date) {
            Some(d) => d,
            None => return None,
        };

        // Date must be within tolerance (±1 day)
        let date_diff = (date1 - date2).num_days().abs();
        if date_diff > self.fuzzy_date_tolerance_days {
            return None;
        }

        // Amount must be within tolerance (±$0.50)
        let amount_diff = (tx1.amount_numeric - tx2.amount_numeric).abs();
        if amount_diff > self.fuzzy_amount_tolerance {
            return None;
        }

        // Merchant must be similar
        let merchant1_lower = tx1.merchant.to_lowercase();
        let merchant2_lower = tx2.merchant.to_lowercase();

        // Strategy 1: One contains the other
        let contains_match = merchant1_lower.contains(&merchant2_lower)
            || merchant2_lower.contains(&merchant1_lower);

        // Strategy 2: Share common word (>= 4 chars, excluding numbers)
        let merchant1_words: Vec<&str> = merchant1_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() >= 4 && !w.chars().all(|c| c.is_numeric()))
            .collect();

        let merchant2_words: Vec<&str> = merchant2_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() >= 4 && !w.chars().all(|c| c.is_numeric()))
            .collect();

        let has_common_word = merchant1_words.iter()
            .any(|w1| merchant2_words.iter().any(|w2| w1 == w2));

        if !contains_match && !has_common_word {
            return None;
        }

        // Calculate confidence based on how close the match is
        let date_score = 1.0 - (date_diff as f64 / (self.fuzzy_date_tolerance_days as f64 + 1.0));
        let amount_score = 1.0 - (amount_diff / (self.fuzzy_amount_tolerance + 0.01));
        let merchant_score = if merchant1_lower == merchant2_lower {
            1.0
        } else {
            0.85  // Similar but not exact
        };

        // Weighted average: date 30%, amount 40%, merchant 30%
        let confidence = (date_score * 0.3 + amount_score * 0.4 + merchant_score * 0.3)
            .max(self.fuzzy_match_threshold);

        Some(DuplicateMatch {
            tx1_index: i,
            tx2_index: j,
            confidence,
            strategy: MatchStrategy::FuzzyMatch,
            reason: format!(
                "Fuzzy match: {} ≈ {} | ${:.2} ≈ ${:.2} | {} ≈ {}",
                tx1.date, tx2.date,
                tx1.amount_numeric.abs(), tx2.amount_numeric.abs(),
                tx1.merchant, tx2.merchant
            ),
        })
    }

    /// Parse date from string (supports MM/DD/YYYY and YYYY-MM-DD)
    fn parse_date(&self, date_str: &str) -> Option<NaiveDate> {
        // Try MM/DD/YYYY
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%m/%d/%Y") {
            return Some(date);
        }

        // Try YYYY-MM-DD
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            return Some(date);
        }

        None
    }
}

impl Default for DeduplicationEngine {
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

    fn create_test_transaction(
        date: &str,
        amount: f64,
        merchant: &str,
        tx_type: &str,
    ) -> Transaction {
        Transaction {
            date: date.to_string(),
            description: "Test transaction".to_string(),
            amount_original: format!("${:.2}", amount),
            amount_numeric: amount,
            transaction_type: tx_type.to_string(),
            category: "Test".to_string(),
            merchant: merchant.to_string(),
            currency: "USD".to_string(),
            account_name: "Test Account".to_string(),
            account_number: "1234".to_string(),
            bank: "Test Bank".to_string(),
            source_file: "test.csv".to_string(),
            line_number: "1".to_string(),
            classification_notes: "".to_string(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_exact_match() {
        let engine = DeduplicationEngine::new();

        let tx1 = create_test_transaction("12/25/2024", 45.99, "Starbucks", "GASTO");
        let tx2 = create_test_transaction("12/25/2024", 45.99, "Starbucks", "GASTO");

        let transactions = vec![tx1, tx2];
        let matches = engine.find_duplicates(&transactions);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].strategy, MatchStrategy::ExactMatch);
        assert!(matches[0].confidence >= 0.95);
    }

    #[test]
    fn test_exact_match_case_insensitive() {
        let engine = DeduplicationEngine::new();

        let tx1 = create_test_transaction("12/25/2024", 45.99, "STARBUCKS", "GASTO");
        let tx2 = create_test_transaction("12/25/2024", 45.99, "starbucks", "GASTO");

        let transactions = vec![tx1, tx2];
        let matches = engine.find_duplicates(&transactions);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].strategy, MatchStrategy::ExactMatch);
    }

    #[test]
    fn test_fuzzy_match_date_tolerance() {
        let engine = DeduplicationEngine::new();

        // Same amount and merchant, but 1 day apart
        let tx1 = create_test_transaction("12/25/2024", 45.99, "Starbucks", "GASTO");
        let tx2 = create_test_transaction("12/26/2024", 45.99, "Starbucks", "GASTO");

        let transactions = vec![tx1, tx2];
        let matches = engine.find_duplicates(&transactions);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].strategy, MatchStrategy::FuzzyMatch);
        assert!(matches[0].confidence >= 0.70);
    }

    #[test]
    fn test_fuzzy_match_amount_tolerance() {
        let engine = DeduplicationEngine::new();

        // Same date and merchant, but amount differs by $0.26
        let tx1 = create_test_transaction("12/25/2024", 45.99, "Starbucks", "GASTO");
        let tx2 = create_test_transaction("12/25/2024", 46.25, "Starbucks", "GASTO");

        let transactions = vec![tx1, tx2];
        let matches = engine.find_duplicates(&transactions);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].strategy, MatchStrategy::FuzzyMatch);
    }

    #[test]
    fn test_fuzzy_match_merchant_similarity() {
        let engine = DeduplicationEngine::new();

        // Same date and amount, similar merchant
        let tx1 = create_test_transaction("12/25/2024", 45.99, "STARBUCKS #4521", "GASTO");
        let tx2 = create_test_transaction("12/25/2024", 45.99, "Starbucks Coffee", "GASTO");

        let transactions = vec![tx1, tx2];
        let matches = engine.find_duplicates(&transactions);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].strategy, MatchStrategy::FuzzyMatch);
    }

    #[test]
    fn test_transfer_pair_detection() {
        let engine = DeduplicationEngine::new();

        // Transfer pair: same date, opposite amounts, both TRASPASO
        let tx1 = create_test_transaction("12/25/2024", -100.00, "Transfer to Wise", "TRASPASO");
        let tx2 = create_test_transaction("12/25/2024", 100.00, "Transfer from BofA", "TRASPASO");

        let transactions = vec![tx1, tx2];
        let matches = engine.find_duplicates(&transactions);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].strategy, MatchStrategy::TransferPair);
        assert!(matches[0].confidence >= 0.90);
    }

    #[test]
    fn test_no_match_different_amounts() {
        let engine = DeduplicationEngine::new();

        // Different amounts (beyond tolerance)
        let tx1 = create_test_transaction("12/25/2024", 45.99, "Starbucks", "GASTO");
        let tx2 = create_test_transaction("12/25/2024", 50.00, "Starbucks", "GASTO");

        let transactions = vec![tx1, tx2];
        let matches = engine.find_duplicates(&transactions);

        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_no_match_different_dates() {
        let engine = DeduplicationEngine::new();

        // Dates too far apart (2 days)
        let tx1 = create_test_transaction("12/25/2024", 45.99, "Starbucks", "GASTO");
        let tx2 = create_test_transaction("12/27/2024", 45.99, "Starbucks", "GASTO");

        let transactions = vec![tx1, tx2];
        let matches = engine.find_duplicates(&transactions);

        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_no_match_different_merchants() {
        let engine = DeduplicationEngine::new();

        // Completely different merchants
        let tx1 = create_test_transaction("12/25/2024", 45.99, "Starbucks", "GASTO");
        let tx2 = create_test_transaction("12/25/2024", 45.99, "Amazon", "GASTO");

        let transactions = vec![tx1, tx2];
        let matches = engine.find_duplicates(&transactions);

        assert_eq!(matches.len(), 0);
    }
}
