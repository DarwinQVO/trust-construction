// 📐 Shape Layer - Schema Validation
// Validates transactions against schemas and contexts

use crate::db::Transaction;
use crate::attributes::{AttributeRegistry, AttributeType};
use anyhow::{Result, anyhow};
use serde_json::Value;

// ============================================================================
// CONTEXT TYPES
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Context {
    /// For UI display - requires user-friendly data
    UI,
    /// For audit/compliance - requires full provenance
    Audit,
    /// For financial reports - requires complete financial data
    Report,
    /// For import/parsing - minimal data + provenance
    Import,
    /// For manual verification - requires review data
    Verification,
    /// For ML training - requires verified, complete data
    MLTraining,
    /// For data quality checks - requires all fields
    Quality,
}

impl Context {
    pub fn name(&self) -> &str {
        match self {
            Context::UI => "UI",
            Context::Audit => "Audit",
            Context::Report => "Report",
            Context::Import => "Import",
            Context::Verification => "Verification",
            Context::MLTraining => "MLTraining",
            Context::Quality => "Quality",
        }
    }
}

// ============================================================================
// VALIDATION RESULT
// ============================================================================

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub context: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.context, self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

pub type ValidationResult = Result<(), Vec<ValidationError>>;

// ============================================================================
// SCHEMA VALIDATOR
// ============================================================================

pub struct SchemaValidator {
    registry: AttributeRegistry,
}

impl SchemaValidator {
    pub fn new() -> Self {
        SchemaValidator {
            registry: AttributeRegistry::new(),
        }
    }
    
    /// Validate a transaction against core Transaction schema
    pub fn validate_transaction(&self, tx: &Transaction) -> ValidationResult {
        let mut errors = Vec::new();
        
        // Required core attributes
        if tx.date.is_empty() {
            errors.push(ValidationError {
                field: "date".to_string(),
                message: "Required field is empty".to_string(),
                context: "Transaction".to_string(),
            });
        }
        
        if tx.description.is_empty() {
            errors.push(ValidationError {
                field: "description".to_string(),
                message: "Required field is empty".to_string(),
                context: "Transaction".to_string(),
            });
        }
        
        if tx.source_file.is_empty() {
            errors.push(ValidationError {
                field: "source_file".to_string(),
                message: "Required field is empty".to_string(),
                context: "Transaction".to_string(),
            });
        }
        
        if tx.line_number.is_empty() {
            errors.push(ValidationError {
                field: "line_number".to_string(),
                message: "Required field is empty".to_string(),
                context: "Transaction".to_string(),
            });
        }
        
        // Validate confidence_score if present
        if let Some(score) = tx.metadata.get("confidence_score") {
            if let Some(score_val) = score.as_f64() {
                if score_val < 0.0 || score_val > 1.0 {
                    errors.push(ValidationError {
                        field: "confidence_score".to_string(),
                        message: format!("Must be between 0.0 and 1.0, got {}", score_val),
                        context: "Transaction".to_string(),
                    });
                }
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Validate transaction against specific context requirements
    pub fn validate_context(&self, tx: &Transaction, context: Context) -> ValidationResult {
        let mut errors = Vec::new();
        let context_name = context.name();
        
        match context {
            Context::UI => {
                // UI requires: date, merchant, amount, transaction_type
                if tx.date.is_empty() {
                    errors.push(ValidationError {
                        field: "date".to_string(),
                        message: "Required for UI display".to_string(),
                        context: context_name.to_string(),
                    });
                }
                
                if tx.merchant.is_empty() {
                    errors.push(ValidationError {
                        field: "merchant".to_string(),
                        message: "Required for UI display".to_string(),
                        context: context_name.to_string(),
                    });
                }
                
                if tx.transaction_type.is_empty() {
                    errors.push(ValidationError {
                        field: "transaction_type".to_string(),
                        message: "Required for UI display".to_string(),
                        context: context_name.to_string(),
                    });
                }
            },
            
            Context::Audit => {
                // Audit requires: source_file, source_line, extracted_at, parser_version
                if tx.source_file.is_empty() {
                    errors.push(ValidationError {
                        field: "source_file".to_string(),
                        message: "Required for audit trail".to_string(),
                        context: context_name.to_string(),
                    });
                }
                
                if tx.line_number.is_empty() {
                    errors.push(ValidationError {
                        field: "line_number".to_string(),
                        message: "Required for audit trail".to_string(),
                        context: context_name.to_string(),
                    });
                }
                
                if !tx.metadata.contains_key("extracted_at") {
                    errors.push(ValidationError {
                        field: "extracted_at".to_string(),
                        message: "Required for audit trail".to_string(),
                        context: context_name.to_string(),
                    });
                }
                
                if !tx.metadata.contains_key("parser_version") {
                    errors.push(ValidationError {
                        field: "parser_version".to_string(),
                        message: "Required for audit trail".to_string(),
                        context: context_name.to_string(),
                    });
                }
            },
            
            Context::Report => {
                // Report requires: date, amount, category, transaction_type
                if tx.date.is_empty() {
                    errors.push(ValidationError {
                        field: "date".to_string(),
                        message: "Required for financial reports".to_string(),
                        context: context_name.to_string(),
                    });
                }
                
                if tx.category.is_empty() {
                    errors.push(ValidationError {
                        field: "category".to_string(),
                        message: "Required for categorized reports".to_string(),
                        context: context_name.to_string(),
                    });
                }
                
                if tx.transaction_type.is_empty() {
                    errors.push(ValidationError {
                        field: "transaction_type".to_string(),
                        message: "Required for financial reports".to_string(),
                        context: context_name.to_string(),
                    });
                }
            },
            
            Context::Verification => {
                // Verification requires: date, description, amount, confidence_score
                if tx.date.is_empty() {
                    errors.push(ValidationError {
                        field: "date".to_string(),
                        message: "Required for verification".to_string(),
                        context: context_name.to_string(),
                    });
                }
                
                if tx.description.is_empty() {
                    errors.push(ValidationError {
                        field: "description".to_string(),
                        message: "Required for verification".to_string(),
                        context: context_name.to_string(),
                    });
                }
                
                if !tx.metadata.contains_key("confidence_score") {
                    errors.push(ValidationError {
                        field: "confidence_score".to_string(),
                        message: "Required to help user decide".to_string(),
                        context: context_name.to_string(),
                    });
                }
            },
            
            Context::MLTraining => {
                // ML Training requires: verified=true, merchant, category, transaction_type
                let verified = tx.metadata.get("verified")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                
                if !verified {
                    errors.push(ValidationError {
                        field: "verified".to_string(),
                        message: "Must be verified for ML training".to_string(),
                        context: context_name.to_string(),
                    });
                }
                
                if tx.merchant.is_empty() {
                    errors.push(ValidationError {
                        field: "merchant".to_string(),
                        message: "Required for ML training".to_string(),
                        context: context_name.to_string(),
                    });
                }
                
                if tx.category.is_empty() {
                    errors.push(ValidationError {
                        field: "category".to_string(),
                        message: "Required for ML training".to_string(),
                        context: context_name.to_string(),
                    });
                }
                
                if tx.transaction_type.is_empty() {
                    errors.push(ValidationError {
                        field: "transaction_type".to_string(),
                        message: "Required for ML training".to_string(),
                        context: context_name.to_string(),
                    });
                }
            },
            
            Context::Quality => {
                // Quality requires: all core fields must exist
                if tx.date.is_empty() {
                    errors.push(ValidationError {
                        field: "date".to_string(),
                        message: "Required for data quality check".to_string(),
                        context: context_name.to_string(),
                    });
                }
                
                if tx.transaction_type.is_empty() {
                    errors.push(ValidationError {
                        field: "transaction_type".to_string(),
                        message: "Required for data quality check".to_string(),
                        context: context_name.to_string(),
                    });
                }
                
                if tx.source_file.is_empty() {
                    errors.push(ValidationError {
                        field: "source_file".to_string(),
                        message: "Required for data quality check".to_string(),
                        context: context_name.to_string(),
                    });
                }
                
                if !tx.metadata.contains_key("extracted_at") {
                    errors.push(ValidationError {
                        field: "extracted_at".to_string(),
                        message: "Required for data quality check".to_string(),
                        context: context_name.to_string(),
                    });
                }
            },
            
            Context::Import => {
                // Import requires: source_file, source_line, extracted_at, description
                if tx.source_file.is_empty() {
                    errors.push(ValidationError {
                        field: "source_file".to_string(),
                        message: "Required for import tracking".to_string(),
                        context: context_name.to_string(),
                    });
                }
                
                if tx.line_number.is_empty() {
                    errors.push(ValidationError {
                        field: "line_number".to_string(),
                        message: "Required for import tracking".to_string(),
                        context: context_name.to_string(),
                    });
                }
                
                if tx.description.is_empty() {
                    errors.push(ValidationError {
                        field: "description".to_string(),
                        message: "Required for import".to_string(),
                        context: context_name.to_string(),
                    });
                }
            },
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Convenience method: validate transaction + context in one call
    pub fn validate(&self, tx: &Transaction, context: Context) -> ValidationResult {
        // First validate core schema
        if let Err(mut schema_errors) = self.validate_transaction(tx) {
            // Then validate context
            if let Err(mut context_errors) = self.validate_context(tx, context) {
                schema_errors.append(&mut context_errors);
            }
            return Err(schema_errors);
        }
        
        // If schema passes, validate context
        self.validate_context(tx, context)
    }
}

impl Default for SchemaValidator {
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
    
    fn create_test_transaction() -> Transaction {
        let mut metadata = HashMap::new();
        metadata.insert("extracted_at".to_string(), serde_json::json!("2024-01-15T10:30:00Z"));
        metadata.insert("parser_version".to_string(), serde_json::json!("test_v1.0"));
        metadata.insert("confidence_score".to_string(), serde_json::json!(0.95));
        
        Transaction {
            date: "01/15/2024".to_string(),
            description: "STARBUCKS".to_string(),
            amount_original: "$45.99".to_string(),
            amount_numeric: 45.99,
            transaction_type: "GASTO".to_string(),
            category: "Restaurants".to_string(),
            merchant: "STARBUCKS".to_string(),
            currency: "USD".to_string(),
            account_name: "Checking".to_string(),
            account_number: "1234".to_string(),
            bank: "BofA".to_string(),
            source_file: "test.csv".to_string(),
            line_number: "23".to_string(),
            classification_notes: String::new(),
            metadata,
        }
    }
    
    #[test]
    fn test_validate_transaction_valid() {
        let validator = SchemaValidator::new();
        let tx = create_test_transaction();
        
        assert!(validator.validate_transaction(&tx).is_ok());
    }
    
    #[test]
    fn test_validate_transaction_missing_required() {
        let validator = SchemaValidator::new();
        let mut tx = create_test_transaction();
        tx.date = String::new();  // Remove required field
        
        let result = validator.validate_transaction(&tx);
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "date");
    }
    
    #[test]
    fn test_validate_context_ui_valid() {
        let validator = SchemaValidator::new();
        let tx = create_test_transaction();
        
        assert!(validator.validate_context(&tx, Context::UI).is_ok());
    }
    
    #[test]
    fn test_validate_context_ui_missing_merchant() {
        let validator = SchemaValidator::new();
        let mut tx = create_test_transaction();
        tx.merchant = String::new();
        
        let result = validator.validate_context(&tx, Context::UI);
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "merchant"));
    }
    
    #[test]
    fn test_validate_context_audit_valid() {
        let validator = SchemaValidator::new();
        let tx = create_test_transaction();
        
        assert!(validator.validate_context(&tx, Context::Audit).is_ok());
    }
    
    #[test]
    fn test_validate_context_audit_missing_provenance() {
        let validator = SchemaValidator::new();
        let mut tx = create_test_transaction();
        tx.metadata.remove("parser_version");
        
        let result = validator.validate_context(&tx, Context::Audit);
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "parser_version"));
    }
    
    #[test]
    fn test_validate_context_report_valid() {
        let validator = SchemaValidator::new();
        let tx = create_test_transaction();
        
        assert!(validator.validate_context(&tx, Context::Report).is_ok());
    }
    
    #[test]
    fn test_validate_context_report_missing_category() {
        let validator = SchemaValidator::new();
        let mut tx = create_test_transaction();
        tx.category = String::new();
        
        let result = validator.validate_context(&tx, Context::Report);
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "category"));
    }
    
    #[test]
    fn test_validate_confidence_score_range() {
        let validator = SchemaValidator::new();
        let mut tx = create_test_transaction();
        
        // Invalid: > 1.0
        tx.metadata.insert("confidence_score".to_string(), serde_json::json!(1.5));
        assert!(validator.validate_transaction(&tx).is_err());
        
        // Invalid: < 0.0
        tx.metadata.insert("confidence_score".to_string(), serde_json::json!(-0.1));
        assert!(validator.validate_transaction(&tx).is_err());
        
        // Valid: in range
        tx.metadata.insert("confidence_score".to_string(), serde_json::json!(0.5));
        assert!(validator.validate_transaction(&tx).is_ok());
    }
    
    #[test]
    fn test_validate_combined() {
        let validator = SchemaValidator::new();
        let tx = create_test_transaction();
        
        // Should pass both schema and UI context
        assert!(validator.validate(&tx, Context::UI).is_ok());
    }
    
    #[test]
    fn test_validate_ml_training_requires_verified() {
        let validator = SchemaValidator::new();
        let mut tx = create_test_transaction();
        
        // Not verified - should fail
        let result = validator.validate_context(&tx, Context::MLTraining);
        assert!(result.is_err());
        
        // Add verified flag
        tx.metadata.insert("verified".to_string(), serde_json::json!(true));
        
        // Should now pass
        assert!(validator.validate_context(&tx, Context::MLTraining).is_ok());
    }
}
