// 🏦 Bank Entity - Stable identity + normalization
// Badge 21: Following Rich Hickey's philosophy
//
// "Bank name is a VALUE (can change), Bank UUID is IDENTITY (never changes)"
//
// Problem solved:
// - "BofA", "Bank of America", "BoA" → All same bank entity
// - Renaming doesn't break historical transactions
// - UUID provides stable foreign key for transactions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

// ============================================================================
// BANK TYPE
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BankType {
    /// Checking account
    Checking,

    /// Savings account
    Savings,

    /// Credit card
    CreditCard,

    /// Payment processor (Stripe, Wise, PayPal)
    PaymentProcessor,

    /// Brokerage / Investment account
    Investment,

    /// Unknown / Other
    Unknown,
}

impl BankType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BankType::Checking => "Checking",
            BankType::Savings => "Savings",
            BankType::CreditCard => "Credit Card",
            BankType::PaymentProcessor => "Payment Processor",
            BankType::Investment => "Investment",
            BankType::Unknown => "Unknown",
        }
    }
}

// ============================================================================
// BANK ENTITY
// ============================================================================

/// Bank Entity - Rich Hickey's Identity/Value separation
///
/// Identity: UUID (never changes)
/// Values: canonical_name, aliases, etc. (can change over time)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bank {
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

    /// Alternative names that map to this bank
    /// Example: ["BofA", "BoA", "Bank of America NA"]
    pub aliases: Vec<String>,

    /// Country code (ISO 3166-1 alpha-2)
    pub country: String,

    /// Type of bank/account
    pub bank_type: BankType,

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

impl Bank {
    /// Create new bank entity with UUID
    pub fn new(
        canonical_name: String,
        country: String,
        bank_type: BankType,
    ) -> Self {
        let now = Utc::now();

        Bank {
            id: uuid::Uuid::new_v4().to_string(),
            canonical_name,
            aliases: Vec::new(),
            country,
            bank_type,
            version: 1,
            system_time: now,
            valid_from: now,
            valid_until: None,
            metadata: serde_json::json!({}),
        }
    }

    /// Add an alias to this bank
    pub fn add_alias(&mut self, alias: String) {
        if !self.aliases.contains(&alias) && alias != self.canonical_name {
            self.aliases.push(alias);
        }
    }

    /// Check if a string matches this bank (canonical name or any alias)
    pub fn matches(&self, bank_string: &str) -> bool {
        let lower = bank_string.to_lowercase();

        // Check canonical name
        if self.canonical_name.to_lowercase().contains(&lower)
            || lower.contains(&self.canonical_name.to_lowercase())
        {
            return true;
        }

        // Check aliases
        self.aliases.iter().any(|alias| {
            let alias_lower = alias.to_lowercase();
            alias_lower.contains(&lower) || lower.contains(&alias_lower)
        })
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
    pub fn next_version(&self) -> Bank {
        let now = Utc::now();
        let mut next = self.clone();
        next.version += 1;
        next.valid_from = now;
        next.valid_until = None;
        next
    }
}

// ============================================================================
// BANK REGISTRY
// ============================================================================

/// Registry of all known banks
///
/// Badge 25: Multi-version storage - stores ALL versions, never deletes
///
/// This is a singleton that holds all Bank entities in memory.
/// In production, this would be backed by a database with compound key (id, version).
pub struct BankRegistry {
    /// ALL versions of all banks (append-only, never delete)
    versions: Arc<RwLock<Vec<Bank>>>,
}

impl BankRegistry {
    /// Create new registry with default banks
    pub fn new() -> Self {
        let mut registry = BankRegistry {
            versions: Arc::new(RwLock::new(Vec::new())),
        };

        registry.register_default_banks();
        registry
    }

    /// Initialize with the 5 known banks from our data
    fn register_default_banks(&mut self) {
        // 1. Bank of America
        let mut bofa = Bank::new(
            "Bank of America".to_string(),
            "US".to_string(),
            BankType::Checking,
        );
        bofa.add_alias("BofA".to_string());
        bofa.add_alias("BoA".to_string());
        bofa.add_alias("Bank of America NA".to_string());
        bofa.add_alias("Bank of America N.A.".to_string());
        self.register(bofa);

        // 2. Apple Card
        let mut apple = Bank::new(
            "Apple Card".to_string(),
            "US".to_string(),
            BankType::CreditCard,
        );
        apple.add_alias("AppleCard".to_string());
        apple.add_alias("Apple".to_string());
        self.register(apple);

        // 3. Stripe
        let mut stripe = Bank::new(
            "Stripe".to_string(),
            "US".to_string(),
            BankType::PaymentProcessor,
        );
        stripe.add_alias("Stripe Inc".to_string());
        stripe.add_alias("Stripe Payments".to_string());
        self.register(stripe);

        // 4. Wise (formerly TransferWise)
        let mut wise = Bank::new(
            "Wise".to_string(),
            "UK".to_string(),
            BankType::PaymentProcessor,
        );
        wise.add_alias("TransferWise".to_string());
        wise.add_alias("Wise Payments".to_string());
        self.register(wise);

        // 5. Scotiabank
        let mut scotiabank = Bank::new(
            "Scotiabank".to_string(),
            "CA".to_string(),
            BankType::Checking,
        );
        scotiabank.add_alias("Scotia".to_string());
        scotiabank.add_alias("Bank of Nova Scotia".to_string());
        scotiabank.add_alias("Scotiabank MX".to_string());  // Mexico operations
        self.register(scotiabank);
    }

    /// Register a new bank version (append-only, never overwrites)
    pub fn register(&mut self, bank: Bank) {
        let mut versions = self.versions.write().unwrap();
        versions.push(bank);
    }

    /// Get ALL versions of a bank by ID
    pub fn get_all_versions(&self, id: &str) -> Vec<Bank> {
        let versions = self.versions.read().unwrap();
        versions
            .iter()
            .filter(|b| b.id == id)
            .cloned()
            .collect()
    }

    /// Get current version of a bank by ID
    pub fn get_current_version(&self, id: &str) -> Option<Bank> {
        let versions = self.versions.read().unwrap();
        versions
            .iter()
            .filter(|b| b.id == id && b.is_current())
            .cloned()
            .next()
    }

    /// Get bank as of a specific time (temporal query)
    ///
    /// This is the core of Rich Hickey's philosophy:
    /// "What was the bank's state at time T?"
    pub fn get_bank_at_time(&self, id: &str, as_of: DateTime<Utc>) -> Option<Bank> {
        let versions = self.versions.read().unwrap();
        versions
            .iter()
            .filter(|b| b.id == id)
            .find(|b| {
                b.valid_from <= as_of
                    && (b.valid_until.is_none() || b.valid_until.unwrap() > as_of)
            })
            .cloned()
    }

    /// Update bank (creates new version, expires old version)
    ///
    /// Badge 25: This is true immutability - never delete, only add
    pub fn update_bank<F>(&mut self, id: &str, mut update_fn: F) -> Result<(), String>
    where
        F: FnMut(&mut Bank),
    {
        let now = Utc::now();

        // 1. Find current version
        let current = self
            .get_current_version(id)
            .ok_or_else(|| format!("Bank not found: {}", id))?;

        // 2. Expire current version
        let mut expired = current.clone();
        expired.valid_until = Some(now);

        // 3. Create new version
        let mut next = current.next_version();
        update_fn(&mut next);

        // 4. Replace current with expired + add new version
        {
            let mut versions = self.versions.write().unwrap();

            // Remove the old current version
            versions.retain(|b| !(b.id == id && b.is_current()));

            // Add expired version + new version
            versions.push(expired);
            versions.push(next);
        }

        Ok(())
    }

    /// Find bank by string (searches canonical name and aliases) - returns current version
    pub fn find_by_string(&self, bank_string: &str) -> Option<Bank> {
        let versions = self.versions.read().unwrap();
        versions
            .iter()
            .filter(|b| b.is_current())
            .find(|bank| bank.matches(bank_string))
            .cloned()
    }

    /// Find bank by UUID - returns current version
    pub fn find_by_id(&self, id: &str) -> Option<Bank> {
        self.get_current_version(id)
    }

    /// Get all banks (current versions only)
    pub fn all_banks(&self) -> Vec<Bank> {
        let versions = self.versions.read().unwrap();
        let mut current: Vec<Bank> = versions.iter().filter(|b| b.is_current()).cloned().collect();

        // Deduplicate by id (keep latest by version)
        current.sort_by(|a, b| a.id.cmp(&b.id).then(b.version.cmp(&a.version)));
        current.dedup_by(|a, b| a.id == b.id);

        current
    }

    /// Count total banks (current versions only)
    pub fn count(&self) -> usize {
        self.all_banks().len()
    }

    /// Get banks by type (current versions only)
    pub fn by_type(&self, bank_type: BankType) -> Vec<Bank> {
        self.all_banks()
            .into_iter()
            .filter(|b| b.bank_type == bank_type)
            .collect()
    }

    /// Get banks by country (current versions only)
    pub fn by_country(&self, country: &str) -> Vec<Bank> {
        self.all_banks()
            .into_iter()
            .filter(|b| b.country == country)
            .collect()
    }

    /// Normalize bank string to canonical name
    ///
    /// Example: "BofA" → "Bank of America"
    pub fn normalize(&self, bank_string: &str) -> Option<String> {
        self.find_by_string(bank_string)
            .map(|bank| bank.canonical_name)
    }

    /// Get bank ID for a bank string (for foreign key references)
    ///
    /// This is what Transaction.bank_id should use
    pub fn get_id(&self, bank_string: &str) -> Option<String> {
        self.find_by_string(bank_string).map(|bank| bank.id)
    }
}

impl Default for BankRegistry {
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
    fn test_bank_creation() {
        let bank = Bank::new(
            "Test Bank".to_string(),
            "US".to_string(),
            BankType::Checking,
        );

        assert!(!bank.id.is_empty());
        assert_eq!(bank.canonical_name, "Test Bank");
        assert_eq!(bank.country, "US");
        assert_eq!(bank.bank_type, BankType::Checking);
        assert_eq!(bank.version, 1);
        assert!(bank.is_current());
        assert_eq!(bank.aliases.len(), 0);
    }

    #[test]
    fn test_bank_add_alias() {
        let mut bank = Bank::new(
            "Bank of America".to_string(),
            "US".to_string(),
            BankType::Checking,
        );

        bank.add_alias("BofA".to_string());
        bank.add_alias("BoA".to_string());
        bank.add_alias("BofA".to_string()); // Duplicate - should not add

        assert_eq!(bank.aliases.len(), 2);
        assert!(bank.aliases.contains(&"BofA".to_string()));
        assert!(bank.aliases.contains(&"BoA".to_string()));
    }

    #[test]
    fn test_bank_matches() {
        let mut bank = Bank::new(
            "Bank of America".to_string(),
            "US".to_string(),
            BankType::Checking,
        );
        bank.add_alias("BofA".to_string());
        bank.add_alias("BoA".to_string());

        // Should match canonical name
        assert!(bank.matches("Bank of America"));
        assert!(bank.matches("bank of america")); // Case insensitive
        assert!(bank.matches("Bank of America NA"));

        // Should match aliases
        assert!(bank.matches("BofA"));
        assert!(bank.matches("bofa")); // Case insensitive
        assert!(bank.matches("BoA"));

        // Should not match unrelated strings
        assert!(!bank.matches("Chase"));
        assert!(!bank.matches("Wells Fargo"));
    }

    #[test]
    fn test_bank_registry_initialization() {
        let registry = BankRegistry::new();

        // Should have 5 default banks
        assert_eq!(registry.count(), 5);

        let banks = registry.all_banks();
        let bank_names: Vec<String> = banks.iter().map(|b| b.canonical_name.clone()).collect();

        assert!(bank_names.contains(&"Bank of America".to_string()));
        assert!(bank_names.contains(&"Apple Card".to_string()));
        assert!(bank_names.contains(&"Stripe".to_string()));
        assert!(bank_names.contains(&"Wise".to_string()));
        assert!(bank_names.contains(&"Scotiabank".to_string()));
    }

    #[test]
    fn test_bank_registry_find_by_string() {
        let registry = BankRegistry::new();

        // Find by canonical name
        let bofa = registry.find_by_string("Bank of America");
        assert!(bofa.is_some());
        assert_eq!(bofa.unwrap().canonical_name, "Bank of America");

        // Find by alias
        let bofa2 = registry.find_by_string("BofA");
        assert!(bofa2.is_some());
        assert_eq!(bofa2.unwrap().canonical_name, "Bank of America");

        // Case insensitive
        let bofa3 = registry.find_by_string("bofa");
        assert!(bofa3.is_some());

        // Unknown bank
        let unknown = registry.find_by_string("Chase");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_bank_registry_find_by_id() {
        let registry = BankRegistry::new();

        let bofa = registry.find_by_string("Bank of America").unwrap();
        let bofa_id = bofa.id.clone();

        let found = registry.find_by_id(&bofa_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().canonical_name, "Bank of America");

        let not_found = registry.find_by_id("non-existent-uuid");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_bank_registry_normalize() {
        let registry = BankRegistry::new();

        // Normalize aliases to canonical names
        assert_eq!(
            registry.normalize("BofA"),
            Some("Bank of America".to_string())
        );
        assert_eq!(
            registry.normalize("bofa"),
            Some("Bank of America".to_string())
        );
        assert_eq!(
            registry.normalize("TransferWise"),
            Some("Wise".to_string())
        );
        assert_eq!(
            registry.normalize("AppleCard"),
            Some("Apple Card".to_string())
        );

        // Unknown bank returns None
        assert_eq!(registry.normalize("Chase"), None);
    }

    #[test]
    fn test_bank_registry_get_id() {
        let registry = BankRegistry::new();

        // Get UUID for bank string
        let bofa_id = registry.get_id("BofA");
        assert!(bofa_id.is_some());

        let bofa_id2 = registry.get_id("Bank of America");
        assert!(bofa_id2.is_some());

        // Same bank should give same ID
        assert_eq!(bofa_id, bofa_id2);

        // Unknown bank
        let unknown_id = registry.get_id("Chase");
        assert!(unknown_id.is_none());
    }

    #[test]
    fn test_bank_registry_by_type() {
        let registry = BankRegistry::new();

        let checking = registry.by_type(BankType::Checking);
        assert_eq!(checking.len(), 2); // BofA, Scotiabank

        let credit_cards = registry.by_type(BankType::CreditCard);
        assert_eq!(credit_cards.len(), 1); // Apple Card

        let processors = registry.by_type(BankType::PaymentProcessor);
        assert_eq!(processors.len(), 2); // Stripe, Wise
    }

    #[test]
    fn test_bank_registry_by_country() {
        let registry = BankRegistry::new();

        let us_banks = registry.by_country("US");
        assert_eq!(us_banks.len(), 3); // BofA, Apple, Stripe

        let uk_banks = registry.by_country("UK");
        assert_eq!(uk_banks.len(), 1); // Wise

        let ca_banks = registry.by_country("CA");
        assert_eq!(ca_banks.len(), 1); // Scotiabank
    }

    #[test]
    fn test_bank_versioning() {
        let bank = Bank::new(
            "Test Bank".to_string(),
            "US".to_string(),
            BankType::Checking,
        );

        let original_version = bank.version;
        let original_valid_from = bank.valid_from;

        // Create next version
        let mut next = bank.next_version();

        assert_eq!(next.version, original_version + 1);
        assert!(next.valid_from > original_valid_from);
        assert!(next.is_current());
        assert_eq!(next.id, bank.id); // Identity remains the same!
    }

    #[test]
    fn test_bank_all_names() {
        let mut bank = Bank::new(
            "Bank of America".to_string(),
            "US".to_string(),
            BankType::Checking,
        );
        bank.add_alias("BofA".to_string());
        bank.add_alias("BoA".to_string());

        let all_names = bank.all_names();
        assert_eq!(all_names.len(), 3);
        assert!(all_names.contains(&"Bank of America".to_string()));
        assert!(all_names.contains(&"BofA".to_string()));
        assert!(all_names.contains(&"BoA".to_string()));
    }

    // ========================================================================
    // BADGE 25: TEMPORAL PERSISTENCE TESTS
    // ========================================================================

    #[test]
    fn test_multi_version_storage() {
        let mut registry = BankRegistry::new();

        // Create a custom bank
        let bank = Bank::new("Test Bank".to_string(), "US".to_string(), BankType::Checking);
        let bank_id = bank.id.clone();
        registry.register(bank);

        // Version 1 exists
        assert_eq!(registry.get_all_versions(&bank_id).len(), 1);

        // Update: change country
        registry
            .update_bank(&bank_id, |b| {
                b.country = "CA".to_string();
            })
            .unwrap();

        // Now we have 2 versions (original + updated)
        let versions = registry.get_all_versions(&bank_id);
        assert_eq!(versions.len(), 2);

        // Version 1 is expired
        assert!(versions[0].valid_until.is_some());
        assert_eq!(versions[0].version, 1);

        // Version 2 is current
        assert!(versions[1].valid_until.is_none());
        assert_eq!(versions[1].version, 2);
    }

    #[test]
    fn test_temporal_query() {
        use chrono::Duration;

        let mut registry = BankRegistry::new();

        let bank = Bank::new("Test Bank".to_string(), "US".to_string(), BankType::Checking);
        let bank_id = bank.id.clone();
        let t0 = Utc::now();

        registry.register(bank);

        // Wait a bit and update
        std::thread::sleep(std::time::Duration::from_millis(10));
        let t1 = Utc::now();

        registry
            .update_bank(&bank_id, |b| {
                b.country = "CA".to_string();
            })
            .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));
        let t2 = Utc::now();

        // Query at t0 (before first version) - should return None
        let before = t0 - Duration::seconds(1);
        assert!(registry.get_bank_at_time(&bank_id, before).is_none());

        // Query at t1 (after first version, before update) - should return version 1
        let at_t1 = registry.get_bank_at_time(&bank_id, t1).unwrap();
        assert_eq!(at_t1.version, 1);
        assert_eq!(at_t1.country, "US");

        // Query at t2 (after update) - should return version 2
        let at_t2 = registry.get_bank_at_time(&bank_id, t2).unwrap();
        assert_eq!(at_t2.version, 2);
        assert_eq!(at_t2.country, "CA");
    }

    #[test]
    fn test_update_preserves_history() {
        let mut registry = BankRegistry::new();

        let bank = Bank::new("Test Bank".to_string(), "US".to_string(), BankType::Checking);
        let bank_id = bank.id.clone();
        registry.register(bank);

        // Original state
        let v1 = registry.get_current_version(&bank_id).unwrap();
        assert_eq!(v1.country, "US");
        assert_eq!(v1.aliases.len(), 0);

        // Update 1: Change country
        registry
            .update_bank(&bank_id, |b| {
                b.country = "CA".to_string();
            })
            .unwrap();

        let v2 = registry.get_current_version(&bank_id).unwrap();
        assert_eq!(v2.country, "CA");
        assert_eq!(v2.version, 2);

        // Update 2: Add alias
        registry
            .update_bank(&bank_id, |b| {
                b.add_alias("TB".to_string());
            })
            .unwrap();

        let v3 = registry.get_current_version(&bank_id).unwrap();
        assert_eq!(v3.country, "CA");
        assert_eq!(v3.aliases.len(), 1);
        assert_eq!(v3.version, 3);

        // CRITICAL: All 3 versions exist
        let all_versions = registry.get_all_versions(&bank_id);
        assert_eq!(all_versions.len(), 3);

        // Version 1: US, no aliases
        assert_eq!(all_versions[0].country, "US");
        assert_eq!(all_versions[0].aliases.len(), 0);

        // Version 2: CA, no aliases
        assert_eq!(all_versions[1].country, "CA");
        assert_eq!(all_versions[1].aliases.len(), 0);

        // Version 3: CA, 1 alias
        assert_eq!(all_versions[2].country, "CA");
        assert_eq!(all_versions[2].aliases.len(), 1);
    }

    #[test]
    fn test_update_expires_previous_version() {
        let mut registry = BankRegistry::new();

        let bank = Bank::new("Test Bank".to_string(), "US".to_string(), BankType::Checking);
        let bank_id = bank.id.clone();
        registry.register(bank);

        // Before update: version 1 is current
        let v1_before = registry.get_current_version(&bank_id).unwrap();
        assert!(v1_before.valid_until.is_none());

        // Update
        registry
            .update_bank(&bank_id, |b| {
                b.country = "CA".to_string();
            })
            .unwrap();

        // After update: version 1 is expired
        let versions = registry.get_all_versions(&bank_id);
        let v1_after = versions.iter().find(|b| b.version == 1).unwrap();
        assert!(v1_after.valid_until.is_some());

        // Version 2 is current
        let v2 = versions.iter().find(|b| b.version == 2).unwrap();
        assert!(v2.valid_until.is_none());
    }

    #[test]
    fn test_identity_persists_across_versions() {
        let mut registry = BankRegistry::new();

        let bank = Bank::new("Test Bank".to_string(), "US".to_string(), BankType::Checking);
        let bank_id = bank.id.clone();
        registry.register(bank);

        // Update multiple times
        for i in 0..5 {
            registry
                .update_bank(&bank_id, |b| {
                    b.country = format!("Country {}", i);
                })
                .unwrap();
        }

        // All versions have same ID (identity persists)
        let versions = registry.get_all_versions(&bank_id);
        assert_eq!(versions.len(), 6); // Original + 5 updates

        for version in versions {
            assert_eq!(version.id, bank_id);
        }
    }

    #[test]
    fn test_get_current_version_returns_latest() {
        let mut registry = BankRegistry::new();

        let bank = Bank::new("Test Bank".to_string(), "US".to_string(), BankType::Checking);
        let bank_id = bank.id.clone();
        registry.register(bank);

        // Update 3 times
        for i in 1..=3 {
            registry
                .update_bank(&bank_id, |b| {
                    b.country = format!("V{}", i);
                })
                .unwrap();
        }

        // get_current_version returns version 4 (original + 3 updates)
        let current = registry.get_current_version(&bank_id).unwrap();
        assert_eq!(current.version, 4);
        assert_eq!(current.country, "V3");
        assert!(current.valid_until.is_none());
    }

    #[test]
    fn test_all_banks_only_returns_current_versions() {
        let mut registry = BankRegistry::new();

        // Create 2 banks
        let bank1 = Bank::new("Bank 1".to_string(), "US".to_string(), BankType::Checking);
        let bank1_id = bank1.id.clone();
        let bank2 = Bank::new("Bank 2".to_string(), "CA".to_string(), BankType::Savings);
        let bank2_id = bank2.id.clone();

        registry.register(bank1);
        registry.register(bank2);

        // Initial: 5 default banks + 2 new banks = 7 current banks
        assert_eq!(registry.all_banks().len(), 7);

        // Update bank1 3 times
        for i in 1..=3 {
            registry
                .update_bank(&bank1_id, |b| {
                    b.country = format!("V{}", i);
                })
                .unwrap();
        }

        // Update bank2 2 times
        for i in 1..=2 {
            registry
                .update_bank(&bank2_id, |b| {
                    b.country = format!("V{}", i);
                })
                .unwrap();
        }

        // Total versions for our 2 test banks: bank1(4) + bank2(3) = 7
        let test_bank_versions: Vec<Bank> = registry
            .versions
            .read()
            .unwrap()
            .iter()
            .filter(|b| b.id == bank1_id || b.id == bank2_id)
            .cloned()
            .collect();
        assert_eq!(test_bank_versions.len(), 7);

        // But all_banks() still returns 7 (5 default + 2 test banks, current versions only)
        assert_eq!(registry.all_banks().len(), 7);

        // Verify that we're only getting current versions
        let all_banks = registry.all_banks();
        for bank in all_banks {
            assert!(bank.is_current());
        }
    }

    #[test]
    fn test_update_nonexistent_bank_fails() {
        let mut registry = BankRegistry::new();

        let result = registry.update_bank("non-existent-id", |b| {
            b.country = "XX".to_string();
        });

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Bank not found"));
    }
}
