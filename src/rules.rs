// 🏷️ Classification Rules - Rules as Data
// Pattern matching and normalization rules for merchant names and categories

use serde::{Deserialize, Serialize};
use anyhow::{Result, Context as AnyhowContext};
use std::fs;
use std::path::Path;

// ============================================================================
// RULE DEFINITION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationRule {
    /// Rule ID for tracking
    pub id: String,

    /// Pattern to match (supports wildcards with *)
    pub pattern: String,

    /// Normalized merchant name
    pub merchant: Option<String>,

    /// Category to assign
    pub category: Option<String>,

    /// Transaction type (GASTO, INGRESO, etc.)
    pub transaction_type: Option<String>,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,

    /// Description/notes about this rule
    pub description: Option<String>,

    /// Priority (higher = applied first)
    #[serde(default = "default_priority")]
    pub priority: i32,
}

fn default_priority() -> i32 {
    0
}

impl ClassificationRule {
    /// Check if pattern matches the given text
    pub fn matches(&self, text: &str) -> bool {
        let pattern_lower = self.pattern.to_lowercase();
        let text_lower = text.to_lowercase();

        if pattern_lower.contains('*') {
            // Wildcard matching
            let parts: Vec<&str> = pattern_lower.split('*').collect();

            if parts.is_empty() {
                return false;
            }

            // Check if text starts with first part
            if !parts[0].is_empty() && !text_lower.starts_with(parts[0]) {
                return false;
            }

            // Check if text ends with last part
            if !parts[parts.len() - 1].is_empty() && !text_lower.ends_with(parts[parts.len() - 1]) {
                return false;
            }

            // Check middle parts appear in order
            let mut current_pos = parts[0].len();
            for i in 1..parts.len() - 1 {
                if parts[i].is_empty() {
                    continue;
                }
                if let Some(pos) = text_lower[current_pos..].find(parts[i]) {
                    current_pos += pos + parts[i].len();
                } else {
                    return false;
                }
            }

            true
        } else {
            // Exact match (case-insensitive)
            text_lower.contains(&pattern_lower)
        }
    }
}

// ============================================================================
// CLASSIFICATION RESULT
// ============================================================================

#[derive(Debug, Clone)]
pub struct ClassificationResult {
    pub merchant: Option<String>,
    pub category: Option<String>,
    pub transaction_type: Option<String>,
    pub confidence: f64,
    pub rule_id: Option<String>,
}

impl Default for ClassificationResult {
    fn default() -> Self {
        ClassificationResult {
            merchant: None,
            category: None,
            transaction_type: None,
            confidence: 0.0,
            rule_id: None,
        }
    }
}

// ============================================================================
// RULE ENGINE
// ============================================================================

pub struct RuleEngine {
    rules: Vec<ClassificationRule>,
}

impl RuleEngine {
    /// Create a new empty rule engine
    pub fn new() -> Self {
        RuleEngine { rules: Vec::new() }
    }

    /// Load rules from JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read rules file: {:?}", path.as_ref()))?;

        let rules: Vec<ClassificationRule> = serde_json::from_str(&content)
            .context("Failed to parse rules JSON")?;

        Ok(RuleEngine::from_rules(rules))
    }

    /// Create engine from a list of rules
    pub fn from_rules(mut rules: Vec<ClassificationRule>) -> Self {
        // Sort by priority (higher first)
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        RuleEngine { rules }
    }

    /// Add a single rule
    pub fn add_rule(&mut self, rule: ClassificationRule) {
        self.rules.push(rule);
        // Re-sort by priority
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Apply rules to classify a merchant/description
    pub fn classify(&self, text: &str) -> ClassificationResult {
        // Find first matching rule (already sorted by priority)
        for rule in &self.rules {
            if rule.matches(text) {
                return ClassificationResult {
                    merchant: rule.merchant.clone(),
                    category: rule.category.clone(),
                    transaction_type: rule.transaction_type.clone(),
                    confidence: rule.confidence,
                    rule_id: Some(rule.id.clone()),
                };
            }
        }

        // No match found
        ClassificationResult::default()
    }

    /// Get number of rules loaded
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for RuleEngine {
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

    #[test]
    fn test_exact_pattern_match() {
        let rule = ClassificationRule {
            id: "test1".to_string(),
            pattern: "STARBUCKS".to_string(),
            merchant: Some("Starbucks".to_string()),
            category: Some("Restaurants".to_string()),
            transaction_type: None,
            confidence: 0.95,
            description: None,
            priority: 0,
        };

        assert!(rule.matches("STARBUCKS COFFEE"));
        assert!(rule.matches("starbucks"));
        assert!(!rule.matches("AMAZON"));
    }

    #[test]
    fn test_wildcard_pattern() {
        let rule = ClassificationRule {
            id: "test2".to_string(),
            pattern: "STARBUCKS*".to_string(),
            merchant: Some("Starbucks".to_string()),
            category: None,
            transaction_type: None,
            confidence: 0.90,
            description: None,
            priority: 0,
        };

        assert!(rule.matches("STARBUCKS COFFEE"));
        assert!(rule.matches("STARBUCKS #4521"));
        assert!(rule.matches("starbucks downtown"));
        assert!(!rule.matches("COFFEE STARBUCKS"));
    }

    #[test]
    fn test_rule_engine_classification() {
        let mut engine = RuleEngine::new();

        engine.add_rule(ClassificationRule {
            id: "starbucks".to_string(),
            pattern: "STARBUCKS*".to_string(),
            merchant: Some("Starbucks".to_string()),
            category: Some("Restaurants".to_string()),
            transaction_type: Some("GASTO".to_string()),
            confidence: 0.95,
            description: Some("Starbucks coffee shop".to_string()),
            priority: 10,
        });

        let result = engine.classify("STARBUCKS COFFEE SHOP");

        assert_eq!(result.merchant, Some("Starbucks".to_string()));
        assert_eq!(result.category, Some("Restaurants".to_string()));
        assert_eq!(result.confidence, 0.95);
        assert_eq!(result.rule_id, Some("starbucks".to_string()));
    }

    #[test]
    fn test_rule_priority() {
        let mut engine = RuleEngine::new();

        // Low priority rule
        engine.add_rule(ClassificationRule {
            id: "general".to_string(),
            pattern: "AMAZON*".to_string(),
            merchant: Some("Amazon".to_string()),
            category: Some("Shopping".to_string()),
            transaction_type: None,
            confidence: 0.80,
            description: None,
            priority: 1,
        });

        // High priority rule
        engine.add_rule(ClassificationRule {
            id: "specific".to_string(),
            pattern: "AMAZON.COM MARKETPLACE".to_string(),
            merchant: Some("Amazon Marketplace".to_string()),
            category: Some("Online Shopping".to_string()),
            transaction_type: None,
            confidence: 0.98,
            description: None,
            priority: 100,
        });

        // Should match high-priority specific rule
        let result = engine.classify("AMAZON.COM MARKETPLACE");
        assert_eq!(result.merchant, Some("Amazon Marketplace".to_string()));
        assert_eq!(result.confidence, 0.98);
    }

    #[test]
    fn test_no_match() {
        let engine = RuleEngine::new();
        let result = engine.classify("UNKNOWN MERCHANT");

        assert_eq!(result.merchant, None);
        assert_eq!(result.category, None);
        assert_eq!(result.confidence, 0.0);
        assert_eq!(result.rule_id, None);
    }
}
