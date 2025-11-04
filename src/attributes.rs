// 🏛️ Semantic Layer - Attribute Registry
// Rich Hickey: "Attributes are independent, not owned by schemas"

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// ATTRIBUTE TYPES
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttributeType {
    String,
    Number,
    DateTime,
    Boolean,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    Required,
    Optional,
    NonEmpty,
    Positive,
    NonZero,
    DateFormat(String),
    Range { min: f64, max: f64 },
    Pattern(String),
}

// ============================================================================
// ATTRIBUTE DEFINITION
// ============================================================================

/// AttributeDefinition - First-class citizen
///
/// Each attribute exists independently of any schema or entity.
/// Attributes describe WHAT something means, not WHERE it's used.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeDefinition {
    /// Unique ID (e.g., "attr:date")
    pub id: String,
    
    /// Human-readable name (e.g., "date")
    pub name: String,
    
    /// Type of value this attribute holds
    pub type_: AttributeType,
    
    /// What does this attribute mean?
    pub description: String,
    
    /// How should this be validated?
    pub validation_rules: Vec<ValidationRule>,
    
    /// Where does this value come from?
    pub provenance_info: String,
    
    /// Optional: Default value
    pub default_value: Option<serde_json::Value>,
    
    /// Optional: Example values
    pub examples: Vec<String>,
}

impl AttributeDefinition {
    /// Create a new attribute definition
    pub fn new(id: impl Into<String>, name: impl Into<String>, type_: AttributeType) -> Self {
        AttributeDefinition {
            id: id.into(),
            name: name.into(),
            type_,
            description: String::new(),
            validation_rules: Vec::new(),
            provenance_info: String::new(),
            default_value: None,
            examples: Vec::new(),
        }
    }
    
    /// Builder: add description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
    
    /// Builder: add validation rule
    pub fn with_validation(mut self, rule: ValidationRule) -> Self {
        self.validation_rules.push(rule);
        self
    }
    
    /// Builder: add provenance info
    pub fn with_provenance(mut self, info: impl Into<String>) -> Self {
        self.provenance_info = info.into();
        self
    }
    
    /// Builder: add example
    pub fn with_example(mut self, example: impl Into<String>) -> Self {
        self.examples.push(example.into());
        self
    }
}

// ============================================================================
// ATTRIBUTE REGISTRY
// ============================================================================

/// AttributeRegistry - Catalog of all attributes
///
/// The registry is the single source of truth for:
/// - What attributes exist
/// - What they mean
/// - How they should be validated
/// - Where they come from
///
/// Rich Hickey: "Attributes are not owned by schemas, they're referenced"
pub struct AttributeRegistry {
    attributes: HashMap<String, AttributeDefinition>,
}

impl AttributeRegistry {
    /// Create a new registry with all core financial attributes
    pub fn new() -> Self {
        let mut registry = AttributeRegistry {
            attributes: HashMap::new(),
        };
        
        registry.register_core_attributes();
        registry
    }
    
    /// Register all core financial transaction attributes
    fn register_core_attributes(&mut self) {
        // ====================================================================
        // TEMPORAL ATTRIBUTES
        // ====================================================================
        
        self.register(
            AttributeDefinition::new("attr:date", "date", AttributeType::DateTime)
                .with_description("Transaction date - when the transaction occurred")
                .with_validation(ValidationRule::Required)
                .with_validation(ValidationRule::DateFormat("MM/DD/YYYY or YYYY-MM-DD".to_string()))
                .with_provenance("Extracted from source document")
                .with_example("01/15/2024")
                .with_example("2024-01-15")
        );
        
        self.register(
            AttributeDefinition::new("attr:extracted_at", "extracted_at", AttributeType::DateTime)
                .with_description("When this data was extracted from source")
                .with_validation(ValidationRule::Required)
                .with_provenance("Set by parser at extraction time")
                .with_example("2024-01-15T10:30:00Z")
        );
        
        // ====================================================================
        // MONETARY ATTRIBUTES
        // ====================================================================
        
        self.register(
            AttributeDefinition::new("attr:amount", "amount", AttributeType::Number)
                .with_description("Transaction amount in USD")
                .with_validation(ValidationRule::Required)
                .with_validation(ValidationRule::NonZero)
                .with_provenance("Extracted and normalized from source")
                .with_example("45.99")
                .with_example("-120.50")
        );
        
        self.register(
            AttributeDefinition::new("attr:amount_original", "amount_original", AttributeType::String)
                .with_description("Original amount string from source (before parsing)")
                .with_provenance("Raw value from source document")
                .with_example("-$45.99")
                .with_example("120.50 USD")
        );
        
        self.register(
            AttributeDefinition::new("attr:currency", "currency", AttributeType::String)
                .with_description("Currency code")
                .with_validation(ValidationRule::Pattern("^[A-Z]{3}$".to_string()))
                .with_provenance("Extracted from source or inferred")
                .with_example("USD")
                .with_example("EUR")
                .with_example("MXN")
        );
        
        // ====================================================================
        // DESCRIPTIVE ATTRIBUTES
        // ====================================================================
        
        self.register(
            AttributeDefinition::new("attr:description", "description", AttributeType::String)
                .with_description("Transaction description from source")
                .with_validation(ValidationRule::Required)
                .with_validation(ValidationRule::NonEmpty)
                .with_provenance("Raw description from source document")
                .with_example("STARBUCKS STORE #12345")
        );
        
        self.register(
            AttributeDefinition::new("attr:merchant", "merchant", AttributeType::String)
                .with_description("Extracted merchant name")
                .with_provenance("Extracted via pattern matching from description")
                .with_example("STARBUCKS")
                .with_example("AMAZON")
        );
        
        // ====================================================================
        // CLASSIFICATION ATTRIBUTES
        // ====================================================================
        
        self.register(
            AttributeDefinition::new("attr:transaction_type", "transaction_type", AttributeType::String)
                .with_description("Transaction type classification")
                .with_validation(ValidationRule::Pattern("^(GASTO|INGRESO|PAGO_TARJETA|TRASPASO)$".to_string()))
                .with_provenance("Classified by parser type classifier")
                .with_example("GASTO")
                .with_example("INGRESO")
        );
        
        self.register(
            AttributeDefinition::new("attr:category", "category", AttributeType::String)
                .with_description("Transaction category")
                .with_provenance("Classified by category rules or ML")
                .with_example("Restaurants")
                .with_example("Transportation")
        );
        
        // ====================================================================
        // PROVENANCE ATTRIBUTES
        // ====================================================================
        
        self.register(
            AttributeDefinition::new("attr:source_file", "source_file", AttributeType::String)
                .with_description("Original source file name")
                .with_validation(ValidationRule::Required)
                .with_validation(ValidationRule::NonEmpty)
                .with_provenance("Parser sets this from filename")
                .with_example("bofa_march_2024.csv")
        );
        
        self.register(
            AttributeDefinition::new("attr:source_line", "source_line", AttributeType::Number)
                .with_description("Line number in source file")
                .with_validation(ValidationRule::Required)
                .with_validation(ValidationRule::Positive)
                .with_provenance("Parser tracks line number during parsing")
                .with_example("23")
        );
        
        self.register(
            AttributeDefinition::new("attr:parser_version", "parser_version", AttributeType::String)
                .with_description("Version of parser that extracted this data")
                .with_provenance("Parser version string")
                .with_example("bofa_parser_v1.0")
        );
        
        // ====================================================================
        // ACCOUNT ATTRIBUTES
        // ====================================================================
        
        self.register(
            AttributeDefinition::new("attr:account_name", "account_name", AttributeType::String)
                .with_description("Account name")
                .with_provenance("From source or inferred")
                .with_example("Checking Account")
        );
        
        self.register(
            AttributeDefinition::new("attr:account_number", "account_number", AttributeType::String)
                .with_description("Account number (last 4 digits)")
                .with_provenance("From source")
                .with_example("5226")
        );
        
        self.register(
            AttributeDefinition::new("attr:bank", "bank", AttributeType::String)
                .with_description("Bank/financial institution")
                .with_provenance("From source type or filename")
                .with_example("BofA")
                .with_example("AppleCard")
                .with_example("Stripe")
        );
        
        // ====================================================================
        // CONFIDENCE & VERIFICATION ATTRIBUTES
        // ====================================================================
        
        self.register(
            AttributeDefinition::new("attr:confidence_score", "confidence_score", AttributeType::Number)
                .with_description("Confidence score for classification (0.0-1.0)")
                .with_validation(ValidationRule::Range { min: 0.0, max: 1.0 })
                .with_provenance("Calculated by classifier")
                .with_example("0.95")
        );
        
        self.register(
            AttributeDefinition::new("attr:verified", "verified", AttributeType::Boolean)
                .with_description("Whether transaction has been manually verified")
                .with_provenance("Set by user")
                .with_example("true")
                .with_example("false")
        );
        
        self.register(
            AttributeDefinition::new("attr:verified_by", "verified_by", AttributeType::String)
                .with_description("Who verified this transaction")
                .with_provenance("Set when user verifies")
                .with_example("user_123")
        );
        
        self.register(
            AttributeDefinition::new("attr:verified_at", "verified_at", AttributeType::DateTime)
                .with_description("When transaction was verified")
                .with_provenance("Set when user verifies")
                .with_example("2024-03-20T15:30:00Z")
        );
    }
    
    /// Register a new attribute
    pub fn register(&mut self, attr: AttributeDefinition) {
        self.attributes.insert(attr.id.clone(), attr);
    }
    
    /// Get attribute definition by ID
    pub fn get(&self, id: &str) -> Option<&AttributeDefinition> {
        self.attributes.get(id)
    }
    
    /// Get attribute definition by name
    pub fn get_by_name(&self, name: &str) -> Option<&AttributeDefinition> {
        self.attributes.values().find(|attr| attr.name == name)
    }
    
    /// List all attribute IDs
    pub fn list_ids(&self) -> Vec<String> {
        self.attributes.keys().cloned().collect()
    }
    
    /// List all attributes
    pub fn list_all(&self) -> Vec<&AttributeDefinition> {
        self.attributes.values().collect()
    }
    
    /// Count total attributes
    pub fn count(&self) -> usize {
        self.attributes.len()
    }
}

impl Default for AttributeRegistry {
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
    fn test_attribute_registry_creation() {
        let registry = AttributeRegistry::new();
        assert!(registry.count() > 0, "Registry should have core attributes");
    }
    
    #[test]
    fn test_get_attribute_by_id() {
        let registry = AttributeRegistry::new();
        
        let attr = registry.get("attr:date");
        assert!(attr.is_some(), "Should find date attribute");
        assert_eq!(attr.unwrap().name, "date");
    }
    
    #[test]
    fn test_get_attribute_by_name() {
        let registry = AttributeRegistry::new();
        
        let attr = registry.get_by_name("amount");
        assert!(attr.is_some(), "Should find amount attribute by name");
        assert_eq!(attr.unwrap().id, "attr:amount");
    }
    
    #[test]
    fn test_core_attributes_registered() {
        let registry = AttributeRegistry::new();
        
        // Temporal
        assert!(registry.get("attr:date").is_some());
        assert!(registry.get("attr:extracted_at").is_some());
        
        // Monetary
        assert!(registry.get("attr:amount").is_some());
        assert!(registry.get("attr:currency").is_some());
        
        // Descriptive
        assert!(registry.get("attr:description").is_some());
        assert!(registry.get("attr:merchant").is_some());
        
        // Classification
        assert!(registry.get("attr:transaction_type").is_some());
        assert!(registry.get("attr:category").is_some());
        
        // Provenance
        assert!(registry.get("attr:source_file").is_some());
        assert!(registry.get("attr:source_line").is_some());
        
        // Confidence
        assert!(registry.get("attr:confidence_score").is_some());
        assert!(registry.get("attr:verified").is_some());
    }
    
    #[test]
    fn test_register_custom_attribute() {
        let mut registry = AttributeRegistry::new();
        let initial_count = registry.count();
        
        let custom_attr = AttributeDefinition::new(
            "attr:custom_field",
            "custom_field",
            AttributeType::String
        ).with_description("A custom field");
        
        registry.register(custom_attr);
        
        assert_eq!(registry.count(), initial_count + 1);
        assert!(registry.get("attr:custom_field").is_some());
    }
    
    #[test]
    fn test_attribute_builder_pattern() {
        let attr = AttributeDefinition::new("attr:test", "test", AttributeType::Number)
            .with_description("Test attribute")
            .with_validation(ValidationRule::Required)
            .with_provenance("Test source")
            .with_example("42");
        
        assert_eq!(attr.description, "Test attribute");
        assert_eq!(attr.validation_rules.len(), 1);
        assert_eq!(attr.provenance_info, "Test source");
        assert_eq!(attr.examples.len(), 1);
    }
}
