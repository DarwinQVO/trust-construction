// 🏪 Merchant Entity - Stable identity + fuzzy matching
// Badge 22: Following Rich Hickey's philosophy
//
// "Merchant name is a VALUE (can change), Merchant UUID is IDENTITY (never changes)"
//
// Problem solved:
// - "STARBUCKS *123", "Starbucks Coffee", "STARBUCKS" → All same merchant entity
// - Location codes normalized ("*123", "#456") → Base merchant
// - Fuzzy matching handles typos and variations
// - UUID provides stable foreign key for transactions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

// ============================================================================
// MERCHANT TYPE
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MerchantType {
    /// Restaurant / Café / Food service
    Restaurant,

    /// Retail store / Shop
    Retail,

    /// Online service (SaaS, subscriptions)
    OnlineService,

    /// Utility (electricity, water, internet)
    Utility,

    /// Transportation (Uber, Lyft, gas stations)
    Transportation,

    /// Entertainment (Netflix, Spotify, movies)
    Entertainment,

    /// Healthcare (doctor, pharmacy)
    Healthcare,

    /// Financial service (bank fees, wire transfers)
    Financial,

    /// Government (taxes, fees)
    Government,

    /// Other / Unknown
    Other,
}

impl MerchantType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MerchantType::Restaurant => "Restaurant",
            MerchantType::Retail => "Retail",
            MerchantType::OnlineService => "Online Service",
            MerchantType::Utility => "Utility",
            MerchantType::Transportation => "Transportation",
            MerchantType::Entertainment => "Entertainment",
            MerchantType::Healthcare => "Healthcare",
            MerchantType::Financial => "Financial",
            MerchantType::Government => "Government",
            MerchantType::Other => "Other",
        }
    }
}

// ============================================================================
// MERCHANT ENTITY
// ============================================================================

/// Merchant Entity - Rich Hickey's Identity/Value separation
///
/// Identity: UUID (never changes)
/// Values: canonical_name, aliases, merchant_type, etc. (can change over time)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Merchant {
    // ========================================================================
    // IDENTITY (Badge 19 - never changes)
    // ========================================================================
    /// Stable identity (UUID) - NEVER changes
    pub id: String,

    // ========================================================================
    // VALUES (can change over time)
    // ========================================================================
    /// Canonical name (the "official" name we use)
    pub canonical_name: String,

    /// Alternative names that map to this merchant
    /// Example: ["STARBUCKS *123", "Starbucks Coffee", "STARBUCKS CORP"]
    pub aliases: Vec<String>,

    /// Type of merchant
    pub merchant_type: MerchantType,

    /// Suggested category (can be None if unknown)
    pub suggested_category: Option<String>,

    // ========================================================================
    // VERSIONING (Badge 19 - temporal tracking)
    // ========================================================================
    pub version: i64,
    pub system_time: DateTime<Utc>,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,

    // ========================================================================
    // METADATA (extensible)
    // ========================================================================
    pub metadata: serde_json::Value,
}

impl Merchant {
    /// Create new merchant entity with UUID
    pub fn new(
        canonical_name: String,
        merchant_type: MerchantType,
        suggested_category: Option<String>,
    ) -> Self {
        let now = Utc::now();

        Merchant {
            id: uuid::Uuid::new_v4().to_string(),
            canonical_name,
            aliases: Vec::new(),
            merchant_type,
            suggested_category,
            version: 1,
            system_time: now,
            valid_from: now,
            valid_until: None,
            metadata: serde_json::json!({}),
        }
    }

    /// Add an alias to this merchant
    pub fn add_alias(&mut self, alias: String) {
        if !self.aliases.contains(&alias) && alias != self.canonical_name {
            self.aliases.push(alias);
        }
    }

    /// Check if a string matches this merchant (with fuzzy matching)
    pub fn matches(&self, merchant_string: &str) -> bool {
        let normalized_input = normalize_merchant_string(merchant_string);
        let normalized_canonical = normalize_merchant_string(&self.canonical_name);

        // Exact match after normalization
        if normalized_input == normalized_canonical {
            return true;
        }

        // Contains match
        if normalized_canonical.contains(&normalized_input)
            || normalized_input.contains(&normalized_canonical)
        {
            return true;
        }

        // Check aliases
        for alias in &self.aliases {
            let normalized_alias = normalize_merchant_string(alias);
            if normalized_input == normalized_alias
                || normalized_alias.contains(&normalized_input)
                || normalized_input.contains(&normalized_alias)
            {
                return true;
            }
        }

        // Fuzzy match (Levenshtein distance)
        if levenshtein_match(&normalized_input, &normalized_canonical, 3) {
            return true;
        }

        false
    }

    /// Get all names (canonical + aliases)
    pub fn all_names(&self) -> Vec<String> {
        let mut names = vec![self.canonical_name.clone()];
        names.extend(self.aliases.clone());
        names
    }

    /// Check if this version is current
    pub fn is_current(&self) -> bool {
        self.valid_until.is_none()
    }

    /// Create next version (for updating values)
    pub fn next_version(&self) -> Merchant {
        let now = Utc::now();
        let mut next = self.clone();
        next.version += 1;
        next.valid_from = now;
        next.valid_until = None;
        next
    }
}

// ============================================================================
// MERCHANT REGISTRY
// ============================================================================

/// Registry of all known merchants
///
/// Badge 25: Multi-version storage - stores ALL versions, never deletes
///
/// This is a singleton that holds all Merchant entities in memory.
/// In production, this would be backed by a database with compound key (id, version).
pub struct MerchantRegistry {
    /// ALL versions of all merchants (append-only, never delete)
    versions: Arc<RwLock<Vec<Merchant>>>,
}

impl MerchantRegistry {
    /// Create new empty registry
    pub fn new() -> Self {
        MerchantRegistry {
            versions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create registry with common merchants pre-loaded
    pub fn with_defaults() -> Self {
        let mut registry = MerchantRegistry::new();
        registry.register_default_merchants();
        registry
    }

    /// Initialize with common merchants
    fn register_default_merchants(&mut self) {
        // 1. Starbucks
        let mut starbucks = Merchant::new(
            "Starbucks".to_string(),
            MerchantType::Restaurant,
            Some("Café".to_string()),
        );
        starbucks.add_alias("STARBUCKS".to_string());
        starbucks.add_alias("Starbucks Coffee".to_string());
        starbucks.add_alias("STARBUCKS CORP".to_string());
        self.register(starbucks);

        // 2. Amazon
        let mut amazon = Merchant::new(
            "Amazon".to_string(),
            MerchantType::Retail,
            Some("Shopping".to_string()),
        );
        amazon.add_alias("AMAZON.COM".to_string());
        amazon.add_alias("Amazon Marketplace".to_string());
        amazon.add_alias("AMZN Mktp".to_string());
        self.register(amazon);

        // 3. Uber
        let mut uber = Merchant::new(
            "Uber".to_string(),
            MerchantType::Transportation,
            Some("Transportation".to_string()),
        );
        uber.add_alias("UBER".to_string());
        uber.add_alias("Uber Trip".to_string());
        uber.add_alias("UBER *TRIP".to_string());
        self.register(uber);

        // 4. Netflix
        let mut netflix = Merchant::new(
            "Netflix".to_string(),
            MerchantType::Entertainment,
            Some("Streaming".to_string()),
        );
        netflix.add_alias("NETFLIX.COM".to_string());
        netflix.add_alias("Netflix Inc".to_string());
        self.register(netflix);

        // 5. Stripe (as a merchant, not a bank)
        let mut stripe_fees = Merchant::new(
            "Stripe Fees".to_string(),
            MerchantType::Financial,
            Some("Business Expense".to_string()),
        );
        stripe_fees.add_alias("Stripe Inc".to_string());
        stripe_fees.add_alias("STRIPE".to_string());
        self.register(stripe_fees);
    }

    /// Register a new merchant version (append-only, never overwrites)
    pub fn register(&mut self, merchant: Merchant) {
        let mut versions = self.versions.write().unwrap();
        versions.push(merchant);
    }

    /// Get ALL versions of a merchant by ID
    pub fn get_all_versions(&self, id: &str) -> Vec<Merchant> {
        let versions = self.versions.read().unwrap();
        versions
            .iter()
            .filter(|m| m.id == id)
            .cloned()
            .collect()
    }

    /// Get current version of a merchant by ID
    pub fn get_current_version(&self, id: &str) -> Option<Merchant> {
        let versions = self.versions.read().unwrap();
        versions
            .iter()
            .filter(|m| m.id == id && m.is_current())
            .cloned()
            .next()
    }

    /// Get merchant as of a specific time (temporal query)
    pub fn get_merchant_at_time(&self, id: &str, as_of: DateTime<Utc>) -> Option<Merchant> {
        let versions = self.versions.read().unwrap();
        versions
            .iter()
            .filter(|m| m.id == id)
            .find(|m| {
                m.valid_from <= as_of
                    && (m.valid_until.is_none() || m.valid_until.unwrap() > as_of)
            })
            .cloned()
    }

    /// Update merchant (creates new version, expires old version)
    pub fn update_merchant<F>(&mut self, id: &str, mut update_fn: F) -> Result<(), String>
    where
        F: FnMut(&mut Merchant),
    {
        let now = Utc::now();

        let current = self
            .get_current_version(id)
            .ok_or_else(|| format!("Merchant not found: {}", id))?;

        let mut expired = current.clone();
        expired.valid_until = Some(now);

        let mut next = current.next_version();
        update_fn(&mut next);

        {
            let mut versions = self.versions.write().unwrap();
            versions.retain(|m| !(m.id == id && m.is_current()));
            versions.push(expired);
            versions.push(next);
        }

        Ok(())
    }

    /// Find merchant by string (searches canonical name and aliases with fuzzy matching) - returns current version
    pub fn find_by_string(&self, merchant_string: &str) -> Option<Merchant> {
        let versions = self.versions.read().unwrap();
        versions
            .iter()
            .filter(|m| m.is_current())
            .find(|merchant| merchant.matches(merchant_string))
            .cloned()
    }

    /// Find merchant by UUID - returns current version
    pub fn find_by_id(&self, id: &str) -> Option<Merchant> {
        self.get_current_version(id)
    }

    /// Get all merchants (current versions only)
    pub fn all_merchants(&self) -> Vec<Merchant> {
        let versions = self.versions.read().unwrap();
        let mut current: Vec<Merchant> = versions.iter().filter(|m| m.is_current()).cloned().collect();

        current.sort_by(|a, b| a.id.cmp(&b.id).then(b.version.cmp(&a.version)));
        current.dedup_by(|a, b| a.id == b.id);

        current
    }

    /// Count total merchants (current versions only)
    pub fn count(&self) -> usize {
        self.all_merchants().len()
    }

    /// Get merchants by type (current versions only)
    pub fn by_type(&self, merchant_type: MerchantType) -> Vec<Merchant> {
        self.all_merchants()
            .into_iter()
            .filter(|m| m.merchant_type == merchant_type)
            .collect()
    }

    /// Normalize merchant string to canonical name
    ///
    /// Example: "STARBUCKS *123" → "Starbucks"
    pub fn normalize(&self, merchant_string: &str) -> Option<String> {
        self.find_by_string(merchant_string)
            .map(|merchant| merchant.canonical_name)
    }

    /// Get merchant ID for a merchant string (for foreign key references)
    ///
    /// This is what Transaction.merchant_id should use
    pub fn get_id(&self, merchant_string: &str) -> Option<String> {
        self.find_by_string(merchant_string).map(|m| m.id)
    }

    /// Get suggested category for a merchant string
    pub fn suggest_category(&self, merchant_string: &str) -> Option<String> {
        self.find_by_string(merchant_string)
            .and_then(|m| m.suggested_category)
    }
}

impl Default for MerchantRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Normalize merchant string for matching
///
/// - Lowercase
/// - Remove location codes (*123, #456) but keep words like *TRIP
/// - Remove common suffixes (Inc, Corp, LLC)
/// - Trim whitespace
fn normalize_merchant_string(s: &str) -> String {
    let mut normalized = s.to_lowercase();

    // Process words: remove pure location codes, clean words with prefixes
    normalized = normalized
        .split_whitespace()
        .filter_map(|word| {
            // If word is pure location code (*123, #456), remove it
            if word.starts_with('*') || word.starts_with('#') {
                let without_prefix = &word[1..];
                // If rest is all digits, it's a location code - remove it
                if without_prefix.chars().all(|c| c.is_ascii_digit()) {
                    return None;
                }
                // Otherwise keep the word without the prefix (*TRIP -> trip)
                Some(without_prefix.to_string())
            } else {
                Some(word.to_string())
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    // Remove common suffixes
    let suffixes = [
        " inc", " corp", " llc", " ltd", " co", " corporation", " company",
        ".com", ".net", ".org",
    ];
    for suffix in &suffixes {
        if normalized.ends_with(suffix) {
            normalized = normalized[..normalized.len() - suffix.len()].to_string();
        }
    }

    normalized.trim().to_string()
}

/// Check if two strings match within Levenshtein distance threshold
///
/// Example:
/// - levenshtein_match("starbucks", "starbuck", 2) = true
/// - levenshtein_match("starbucks", "amazon", 2) = false
fn levenshtein_match(s1: &str, s2: &str, threshold: usize) -> bool {
    let distance = levenshtein_distance(s1, s2);
    distance <= threshold
}

/// Calculate Levenshtein distance between two strings
///
/// Levenshtein distance = minimum number of single-character edits
/// (insertions, deletions, substitutions) to change one string into another
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first row and column
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    // Fill matrix
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };

            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1,      // deletion
                    matrix[i][j - 1] + 1,      // insertion
                ),
                matrix[i - 1][j - 1] + cost,   // substitution
            );
        }
    }

    matrix[len1][len2]
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merchant_creation() {
        let merchant = Merchant::new(
            "Test Merchant".to_string(),
            MerchantType::Retail,
            Some("Shopping".to_string()),
        );

        assert!(!merchant.id.is_empty());
        assert_eq!(merchant.canonical_name, "Test Merchant");
        assert_eq!(merchant.merchant_type, MerchantType::Retail);
        assert_eq!(merchant.suggested_category, Some("Shopping".to_string()));
        assert_eq!(merchant.version, 1);
        assert!(merchant.is_current());
        assert_eq!(merchant.aliases.len(), 0);
    }

    #[test]
    fn test_merchant_add_alias() {
        let mut merchant = Merchant::new(
            "Starbucks".to_string(),
            MerchantType::Restaurant,
            Some("Café".to_string()),
        );

        merchant.add_alias("STARBUCKS".to_string());
        merchant.add_alias("Starbucks Coffee".to_string());
        merchant.add_alias("STARBUCKS".to_string()); // Duplicate - should not add

        assert_eq!(merchant.aliases.len(), 2);
        assert!(merchant.aliases.contains(&"STARBUCKS".to_string()));
        assert!(merchant.aliases.contains(&"Starbucks Coffee".to_string()));
    }

    #[test]
    fn test_normalize_merchant_string() {
        assert_eq!(normalize_merchant_string("STARBUCKS *123"), "starbucks");
        assert_eq!(
            normalize_merchant_string("Amazon.com Inc"),
            "amazon"
        );
        assert_eq!(normalize_merchant_string("UBER *TRIP #456"), "uber trip");
        assert_eq!(
            normalize_merchant_string("Stripe Corporation"),
            "stripe"
        );
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
        assert_eq!(levenshtein_distance("abc", "ab"), 1);
        assert_eq!(levenshtein_distance("abc", "abcd"), 1);
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("starbucks", "starbuck"), 1);
    }

    #[test]
    fn test_levenshtein_match() {
        assert!(levenshtein_match("starbucks", "starbuck", 2));
        assert!(levenshtein_match("starbucks", "starbucks", 0));
        assert!(!levenshtein_match("starbucks", "amazon", 2));
        assert!(levenshtein_match("uber", "ubar", 1));
    }

    #[test]
    fn test_merchant_matches() {
        let mut merchant = Merchant::new(
            "Starbucks".to_string(),
            MerchantType::Restaurant,
            Some("Café".to_string()),
        );
        merchant.add_alias("STARBUCKS".to_string());
        merchant.add_alias("Starbucks Coffee".to_string());

        // Exact matches
        assert!(merchant.matches("Starbucks"));
        assert!(merchant.matches("STARBUCKS"));
        assert!(merchant.matches("Starbucks Coffee"));

        // Normalized matches (location codes removed)
        assert!(merchant.matches("STARBUCKS *123"));
        assert!(merchant.matches("Starbucks #456"));

        // Contains matches
        assert!(merchant.matches("Starbucks Corp"));

        // Fuzzy matches (typos)
        assert!(merchant.matches("Starbuck")); // Missing 's'
        assert!(merchant.matches("Starbuckz")); // Wrong letter

        // Should not match
        assert!(!merchant.matches("Amazon"));
        assert!(!merchant.matches("Uber"));
    }

    #[test]
    fn test_merchant_registry_initialization() {
        let registry = MerchantRegistry::with_defaults();

        // Should have 5 default merchants
        assert_eq!(registry.count(), 5);

        let merchants = registry.all_merchants();
        let merchant_names: Vec<String> =
            merchants.iter().map(|m| m.canonical_name.clone()).collect();

        assert!(merchant_names.contains(&"Starbucks".to_string()));
        assert!(merchant_names.contains(&"Amazon".to_string()));
        assert!(merchant_names.contains(&"Uber".to_string()));
        assert!(merchant_names.contains(&"Netflix".to_string()));
        assert!(merchant_names.contains(&"Stripe Fees".to_string()));
    }

    #[test]
    fn test_merchant_registry_find_by_string() {
        let registry = MerchantRegistry::with_defaults();

        // Find by canonical name
        let starbucks = registry.find_by_string("Starbucks");
        assert!(starbucks.is_some());
        assert_eq!(starbucks.unwrap().canonical_name, "Starbucks");

        // Find by alias
        let starbucks2 = registry.find_by_string("STARBUCKS");
        assert!(starbucks2.is_some());
        assert_eq!(starbucks2.unwrap().canonical_name, "Starbucks");

        // Find with location code (normalized)
        let starbucks3 = registry.find_by_string("STARBUCKS *123");
        assert!(starbucks3.is_some());
        assert_eq!(starbucks3.unwrap().canonical_name, "Starbucks");

        // Unknown merchant
        let unknown = registry.find_by_string("Target");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_merchant_registry_find_by_id() {
        let registry = MerchantRegistry::with_defaults();

        let starbucks = registry.find_by_string("Starbucks").unwrap();
        let starbucks_id = starbucks.id.clone();

        let found = registry.find_by_id(&starbucks_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().canonical_name, "Starbucks");

        let not_found = registry.find_by_id("non-existent-uuid");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_merchant_registry_normalize() {
        let registry = MerchantRegistry::with_defaults();

        // Normalize aliases to canonical names
        assert_eq!(
            registry.normalize("STARBUCKS *123"),
            Some("Starbucks".to_string())
        );
        assert_eq!(
            registry.normalize("AMAZON.COM"),
            Some("Amazon".to_string())
        );
        assert_eq!(
            registry.normalize("UBER *TRIP"),
            Some("Uber".to_string())
        );
        assert_eq!(
            registry.normalize("Netflix.com"),
            Some("Netflix".to_string())
        );

        // Unknown merchant returns None
        assert_eq!(registry.normalize("Target"), None);
    }

    #[test]
    fn test_merchant_registry_get_id() {
        let registry = MerchantRegistry::with_defaults();

        // Get UUID for merchant string
        let starbucks_id = registry.get_id("STARBUCKS *123");
        assert!(starbucks_id.is_some());

        let starbucks_id2 = registry.get_id("Starbucks");
        assert!(starbucks_id2.is_some());

        // Same merchant should give same ID
        assert_eq!(starbucks_id, starbucks_id2);

        // Unknown merchant
        let unknown_id = registry.get_id("Target");
        assert!(unknown_id.is_none());
    }

    #[test]
    fn test_merchant_registry_suggest_category() {
        let registry = MerchantRegistry::with_defaults();

        assert_eq!(
            registry.suggest_category("STARBUCKS *123"),
            Some("Café".to_string())
        );
        assert_eq!(
            registry.suggest_category("Amazon.com"),
            Some("Shopping".to_string())
        );
        assert_eq!(
            registry.suggest_category("UBER"),
            Some("Transportation".to_string())
        );

        // Unknown merchant
        assert_eq!(registry.suggest_category("Target"), None);
    }

    #[test]
    fn test_merchant_registry_by_type() {
        let registry = MerchantRegistry::with_defaults();

        let restaurants = registry.by_type(MerchantType::Restaurant);
        assert_eq!(restaurants.len(), 1); // Starbucks

        let retail = registry.by_type(MerchantType::Retail);
        assert_eq!(retail.len(), 1); // Amazon

        let transportation = registry.by_type(MerchantType::Transportation);
        assert_eq!(transportation.len(), 1); // Uber

        let entertainment = registry.by_type(MerchantType::Entertainment);
        assert_eq!(entertainment.len(), 1); // Netflix

        let financial = registry.by_type(MerchantType::Financial);
        assert_eq!(financial.len(), 1); // Stripe Fees
    }

    #[test]
    fn test_merchant_versioning() {
        let merchant = Merchant::new(
            "Test Merchant".to_string(),
            MerchantType::Retail,
            Some("Shopping".to_string()),
        );

        let original_version = merchant.version;
        let original_valid_from = merchant.valid_from;

        // Create next version
        let next = merchant.next_version();

        assert_eq!(next.version, original_version + 1);
        assert!(next.valid_from > original_valid_from);
        assert!(next.is_current());
        assert_eq!(next.id, merchant.id); // Identity remains the same!
    }

    #[test]
    fn test_merchant_all_names() {
        let mut merchant = Merchant::new(
            "Starbucks".to_string(),
            MerchantType::Restaurant,
            Some("Café".to_string()),
        );
        merchant.add_alias("STARBUCKS".to_string());
        merchant.add_alias("Starbucks Coffee".to_string());

        let all_names = merchant.all_names();
        assert_eq!(all_names.len(), 3);
        assert!(all_names.contains(&"Starbucks".to_string()));
        assert!(all_names.contains(&"STARBUCKS".to_string()));
        assert!(all_names.contains(&"Starbucks Coffee".to_string()));
    }

    // ========================================================================
    // BADGE 25: TEMPORAL PERSISTENCE TESTS
    // ========================================================================

    #[test]
    fn test_merchant_multi_version_storage() {
        let mut registry = MerchantRegistry::new();

        let merchant = Merchant::new("Test Merchant".to_string(), MerchantType::Retail, None);
        let merchant_id = merchant.id.clone();
        registry.register(merchant);

        assert_eq!(registry.get_all_versions(&merchant_id).len(), 1);

        registry
            .update_merchant(&merchant_id, |m| {
                m.merchant_type = MerchantType::Restaurant;
            })
            .unwrap();

        let versions = registry.get_all_versions(&merchant_id);
        assert_eq!(versions.len(), 2);

        assert!(versions[0].valid_until.is_some());
        assert_eq!(versions[0].version, 1);

        assert!(versions[1].valid_until.is_none());
        assert_eq!(versions[1].version, 2);
    }

    #[test]
    fn test_merchant_temporal_query() {
        use chrono::Duration;

        let mut registry = MerchantRegistry::new();

        let merchant = Merchant::new("Test Merchant".to_string(), MerchantType::Retail, None);
        let merchant_id = merchant.id.clone();
        let t0 = Utc::now();

        registry.register(merchant);

        std::thread::sleep(std::time::Duration::from_millis(10));
        let t1 = Utc::now();

        registry
            .update_merchant(&merchant_id, |m| {
                m.suggested_category = Some("Shopping".to_string());
            })
            .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));
        let t2 = Utc::now();

        let before = t0 - Duration::seconds(1);
        assert!(registry.get_merchant_at_time(&merchant_id, before).is_none());

        let at_t1 = registry.get_merchant_at_time(&merchant_id, t1).unwrap();
        assert_eq!(at_t1.version, 1);
        assert!(at_t1.suggested_category.is_none());

        let at_t2 = registry.get_merchant_at_time(&merchant_id, t2).unwrap();
        assert_eq!(at_t2.version, 2);
        assert_eq!(at_t2.suggested_category, Some("Shopping".to_string()));
    }

    #[test]
    fn test_merchant_update_preserves_history() {
        let mut registry = MerchantRegistry::new();

        let merchant = Merchant::new("Test Merchant".to_string(), MerchantType::Retail, None);
        let merchant_id = merchant.id.clone();
        registry.register(merchant);

        let v1 = registry.get_current_version(&merchant_id).unwrap();
        assert_eq!(v1.merchant_type, MerchantType::Retail);
        assert_eq!(v1.aliases.len(), 0);

        registry
            .update_merchant(&merchant_id, |m| {
                m.merchant_type = MerchantType::Restaurant;
            })
            .unwrap();

        let v2 = registry.get_current_version(&merchant_id).unwrap();
        assert_eq!(v2.merchant_type, MerchantType::Restaurant);
        assert_eq!(v2.version, 2);

        registry
            .update_merchant(&merchant_id, |m| {
                m.add_alias("TM".to_string());
            })
            .unwrap();

        let v3 = registry.get_current_version(&merchant_id).unwrap();
        assert_eq!(v3.merchant_type, MerchantType::Restaurant);
        assert_eq!(v3.aliases.len(), 1);
        assert_eq!(v3.version, 3);

        let all_versions = registry.get_all_versions(&merchant_id);
        assert_eq!(all_versions.len(), 3);

        assert_eq!(all_versions[0].merchant_type, MerchantType::Retail);
        assert_eq!(all_versions[0].aliases.len(), 0);

        assert_eq!(all_versions[1].merchant_type, MerchantType::Restaurant);
        assert_eq!(all_versions[1].aliases.len(), 0);

        assert_eq!(all_versions[2].merchant_type, MerchantType::Restaurant);
        assert_eq!(all_versions[2].aliases.len(), 1);
    }

    #[test]
    fn test_merchant_update_expires_previous_version() {
        let mut registry = MerchantRegistry::new();

        let merchant = Merchant::new("Test Merchant".to_string(), MerchantType::Retail, None);
        let merchant_id = merchant.id.clone();
        registry.register(merchant);

        let v1_before = registry.get_current_version(&merchant_id).unwrap();
        assert!(v1_before.valid_until.is_none());

        registry
            .update_merchant(&merchant_id, |m| {
                m.merchant_type = MerchantType::Restaurant;
            })
            .unwrap();

        let versions = registry.get_all_versions(&merchant_id);
        let v1_after = versions.iter().find(|m| m.version == 1).unwrap();
        assert!(v1_after.valid_until.is_some());

        let v2 = versions.iter().find(|m| m.version == 2).unwrap();
        assert!(v2.valid_until.is_none());
    }

    #[test]
    fn test_merchant_identity_persists_across_versions() {
        let mut registry = MerchantRegistry::new();

        let merchant = Merchant::new("Test Merchant".to_string(), MerchantType::Retail, None);
        let merchant_id = merchant.id.clone();
        registry.register(merchant);

        for i in 0..5 {
            registry
                .update_merchant(&merchant_id, |m| {
                    m.canonical_name = format!("Merchant {}", i);
                })
                .unwrap();
        }

        let versions = registry.get_all_versions(&merchant_id);
        assert_eq!(versions.len(), 6);

        for version in versions {
            assert_eq!(version.id, merchant_id);
        }
    }

    #[test]
    fn test_merchant_get_current_version_returns_latest() {
        let mut registry = MerchantRegistry::new();

        let merchant = Merchant::new("Test Merchant".to_string(), MerchantType::Retail, None);
        let merchant_id = merchant.id.clone();
        registry.register(merchant);

        for i in 1..=3 {
            registry
                .update_merchant(&merchant_id, |m| {
                    m.canonical_name = format!("V{}", i);
                })
                .unwrap();
        }

        let current = registry.get_current_version(&merchant_id).unwrap();
        assert_eq!(current.version, 4);
        assert_eq!(current.canonical_name, "V3");
        assert!(current.valid_until.is_none());
    }

    #[test]
    fn test_merchant_all_only_returns_current_versions() {
        let mut registry = MerchantRegistry::with_defaults();

        let merchant1 = Merchant::new("Merchant 1".to_string(), MerchantType::Retail, None);
        let merchant1_id = merchant1.id.clone();
        let merchant2 = Merchant::new("Merchant 2".to_string(), MerchantType::Restaurant, None);
        let merchant2_id = merchant2.id.clone();

        registry.register(merchant1);
        registry.register(merchant2);

        let initial_count = registry.all_merchants().len();
        assert_eq!(initial_count, 7); // 5 default + 2 new

        for i in 1..=3 {
            registry
                .update_merchant(&merchant1_id, |m| {
                    m.canonical_name = format!("V{}", i);
                })
                .unwrap();
        }

        for i in 1..=2 {
            registry
                .update_merchant(&merchant2_id, |m| {
                    m.canonical_name = format!("V{}", i);
                })
                .unwrap();
        }

        let test_merchant_versions: Vec<Merchant> = registry
            .versions
            .read()
            .unwrap()
            .iter()
            .filter(|m| m.id == merchant1_id || m.id == merchant2_id)
            .cloned()
            .collect();
        assert_eq!(test_merchant_versions.len(), 7);

        assert_eq!(registry.all_merchants().len(), 7);

        let all_merchants = registry.all_merchants();
        for merchant in all_merchants {
            assert!(merchant.is_current());
        }
    }

    #[test]
    fn test_merchant_update_nonexistent_fails() {
        let mut registry = MerchantRegistry::new();

        let result = registry.update_merchant("non-existent-id", |m| {
            m.canonical_name = "XX".to_string();
        });

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Merchant not found"));
    }
}
