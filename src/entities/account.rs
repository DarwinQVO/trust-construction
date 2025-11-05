// 💳 Account Entity - Stable identity with Bank relationship
// Badge 24: Following Rich Hickey's philosophy (FINAL BADGE!)
//
// "Account name is a VALUE (can change), Account UUID is IDENTITY (never changes)"
//
// Problem solved:
// - Account "BofA Checking *1234" has stable UUID identity
// - Foreign key relationship to Bank entity (bank_id)
// - Renaming doesn't break historical transactions
// - Balance tracking with temporal history
// - UUID provides stable foreign key for transactions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

// ============================================================================
// ACCOUNT TYPE
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AccountType {
    /// Checking account (debit card, daily transactions)
    Checking,

    /// Savings account (interest-bearing)
    Savings,

    /// Credit card (credit line)
    Credit,

    /// Investment account (brokerage, stocks, bonds)
    Investment,

    /// Other / Unknown
    Other,
}

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountType::Checking => "Checking",
            AccountType::Savings => "Savings",
            AccountType::Credit => "Credit",
            AccountType::Investment => "Investment",
            AccountType::Other => "Other",
        }
    }
}

// ============================================================================
// ACCOUNT ENTITY
// ============================================================================

/// Account Entity - Rich Hickey's Identity/Value separation + Bank relationship
///
/// Identity: UUID (never changes)
/// Values: name, account_number, balance, etc. (can change over time)
/// Relationship: bank_id → Bank entity (foreign key)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    // ========================================================================
    // IDENTITY (Badge 19 - never changes)
    // ========================================================================
    /// Stable identity (UUID) - NEVER changes
    pub id: String,

    // ========================================================================
    // VALUES (can change over time)
    // ========================================================================
    /// Account name (e.g., "BofA Checking *1234")
    pub name: String,

    /// Account number (last 4 digits, masked for security)
    /// Example: "*1234", "*5678"
    pub account_number: String,

    /// Bank ID (foreign key to Bank entity)
    pub bank_id: String,

    /// Type of account
    pub account_type: AccountType,

    /// Currency (ISO 4217 code: USD, EUR, MXN, etc.)
    pub currency: String,

    /// Opening balance (when account was created/imported)
    pub opening_balance: f64,

    /// Current balance (updated with each transaction)
    pub current_balance: f64,

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

impl Account {
    /// Create new account entity with UUID
    pub fn new(
        name: String,
        account_number: String,
        bank_id: String,
        account_type: AccountType,
        currency: String,
        opening_balance: f64,
    ) -> Self {
        let now = Utc::now();

        Account {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            account_number,
            bank_id,
            account_type,
            currency,
            opening_balance,
            current_balance: opening_balance,
            version: 1,
            system_time: now,
            valid_from: now,
            valid_until: None,
            metadata: serde_json::json!({}),
        }
    }

    /// Update balance (creates new version)
    pub fn update_balance(&mut self, new_balance: f64) {
        self.current_balance = new_balance;
    }

    /// Get balance change
    pub fn balance_change(&self) -> f64 {
        self.current_balance - self.opening_balance
    }

    /// Check if account has positive balance
    pub fn is_positive(&self) -> bool {
        self.current_balance > 0.0
    }

    /// Check if account is overdrawn (negative balance)
    pub fn is_overdrawn(&self) -> bool {
        self.current_balance < 0.0
    }

    /// Check if this version is current
    pub fn is_current(&self) -> bool {
        self.valid_until.is_none()
    }

    /// Create next version (for updating values)
    pub fn next_version(&self) -> Account {
        let now = Utc::now();
        let mut next = self.clone();
        next.version += 1;
        next.valid_from = now;
        next.valid_until = None;
        next
    }

    /// Mask account number (show only last 4 digits)
    ///
    /// Example: "1234567890" → "*1234"
    pub fn mask_account_number(full_number: &str) -> String {
        if full_number.len() <= 4 {
            return full_number.to_string();
        }
        let last4 = &full_number[full_number.len() - 4..];
        format!("*{}", last4)
    }
}

// ============================================================================
// ACCOUNT REGISTRY
// ============================================================================

/// Registry of all known accounts
///
/// Badge 25: Multi-version storage - stores ALL versions, never deletes
///
/// This is a singleton that holds all Account entities in memory.
/// Maintains relationships with Bank entities via bank_id.
/// In production, this would be backed by a database with compound key (id, version).
pub struct AccountRegistry {
    /// ALL versions of all accounts (append-only, never delete)
    versions: Arc<RwLock<Vec<Account>>>,
}

impl AccountRegistry {
    /// Create new empty registry
    pub fn new() -> Self {
        AccountRegistry {
            versions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a new account version (append-only, never overwrites)
    pub fn register(&mut self, account: Account) {
        let mut versions = self.versions.write().unwrap();
        versions.push(account);
    }

    /// Get ALL versions of an account by ID
    pub fn get_all_versions(&self, id: &str) -> Vec<Account> {
        let versions = self.versions.read().unwrap();
        versions
            .iter()
            .filter(|a| a.id == id)
            .cloned()
            .collect()
    }

    /// Get current version of an account by ID
    pub fn get_current_version(&self, id: &str) -> Option<Account> {
        let versions = self.versions.read().unwrap();
        versions
            .iter()
            .filter(|a| a.id == id && a.is_current())
            .cloned()
            .next()
    }

    /// Get account as of a specific time (temporal query)
    pub fn get_account_at_time(&self, id: &str, as_of: DateTime<Utc>) -> Option<Account> {
        let versions = self.versions.read().unwrap();
        versions
            .iter()
            .filter(|a| a.id == id)
            .find(|a| {
                a.valid_from <= as_of
                    && (a.valid_until.is_none() || a.valid_until.unwrap() > as_of)
            })
            .cloned()
    }

    /// Update account (creates new version, expires old version)
    pub fn update_account<F>(&mut self, id: &str, mut update_fn: F) -> Result<(), String>
    where
        F: FnMut(&mut Account),
    {
        let now = Utc::now();

        let current = self
            .get_current_version(id)
            .ok_or_else(|| format!("Account not found: {}", id))?;

        let mut expired = current.clone();
        expired.valid_until = Some(now);

        let mut next = current.next_version();
        update_fn(&mut next);

        {
            let mut versions = self.versions.write().unwrap();
            versions.retain(|a| !(a.id == id && a.is_current()));
            versions.push(expired);
            versions.push(next);
        }

        Ok(())
    }

    /// Find account by name (exact match, case-insensitive) - returns current version
    pub fn find_by_name(&self, name: &str) -> Option<Account> {
        let versions = self.versions.read().unwrap();
        let lower_name = name.to_lowercase();
        versions
            .iter()
            .filter(|a| a.is_current())
            .find(|acc| acc.name.to_lowercase() == lower_name)
            .cloned()
    }

    /// Find account by UUID - returns current version
    pub fn find_by_id(&self, id: &str) -> Option<Account> {
        self.get_current_version(id)
    }

    /// Find account by account number (last 4 digits) - returns current version
    pub fn find_by_account_number(&self, account_number: &str) -> Option<Account> {
        let versions = self.versions.read().unwrap();
        versions
            .iter()
            .filter(|a| a.is_current())
            .find(|acc| acc.account_number == account_number)
            .cloned()
    }

    /// Get all accounts (current versions only)
    pub fn all_accounts(&self) -> Vec<Account> {
        let versions = self.versions.read().unwrap();
        let mut current: Vec<Account> = versions.iter().filter(|a| a.is_current()).cloned().collect();

        current.sort_by(|a, b| a.id.cmp(&b.id).then(b.version.cmp(&a.version)));
        current.dedup_by(|a, b| a.id == b.id);

        current
    }

    /// Count total accounts (current versions only)
    pub fn count(&self) -> usize {
        self.all_accounts().len()
    }

    /// Get accounts by bank ID (current versions only)
    pub fn by_bank(&self, bank_id: &str) -> Vec<Account> {
        self.all_accounts()
            .into_iter()
            .filter(|acc| acc.bank_id == bank_id)
            .collect()
    }

    /// Get accounts by type (current versions only)
    pub fn by_type(&self, account_type: AccountType) -> Vec<Account> {
        self.all_accounts()
            .into_iter()
            .filter(|acc| acc.account_type == account_type)
            .collect()
    }

    /// Get accounts by currency (current versions only)
    pub fn by_currency(&self, currency: &str) -> Vec<Account> {
        self.all_accounts()
            .into_iter()
            .filter(|acc| acc.currency == currency)
            .collect()
    }

    /// Get account ID for an account name (for foreign key references)
    pub fn get_id(&self, name: &str) -> Option<String> {
        self.find_by_name(name).map(|acc| acc.id)
    }

    /// Calculate total balance across all accounts (current versions only)
    pub fn total_balance(&self) -> f64 {
        self.all_accounts().iter().map(|acc| acc.current_balance).sum()
    }

    /// Calculate total balance by currency (current versions only)
    pub fn total_balance_by_currency(&self, currency: &str) -> f64 {
        self.all_accounts()
            .iter()
            .filter(|acc| acc.currency == currency)
            .map(|acc| acc.current_balance)
            .sum()
    }

    /// Get accounts with positive balance (current versions only)
    pub fn positive_accounts(&self) -> Vec<Account> {
        let accounts = self.all_accounts();
        accounts
            .iter()
            .filter(|acc| acc.is_positive())
            .cloned()
            .collect()
    }

    /// Get accounts with negative balance (overdrawn, current versions only)
    pub fn overdrawn_accounts(&self) -> Vec<Account> {
        self.all_accounts()
            .into_iter()
            .filter(|acc| acc.is_overdrawn())
            .collect()
    }
}

impl Default for AccountRegistry {
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

    fn create_test_bank_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    #[test]
    fn test_account_creation() {
        let bank_id = create_test_bank_id();
        let account = Account::new(
            "Test Checking".to_string(),
            "*1234".to_string(),
            bank_id.clone(),
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );

        assert!(!account.id.is_empty());
        assert_eq!(account.name, "Test Checking");
        assert_eq!(account.account_number, "*1234");
        assert_eq!(account.bank_id, bank_id);
        assert_eq!(account.account_type, AccountType::Checking);
        assert_eq!(account.currency, "USD");
        assert_eq!(account.opening_balance, 1000.0);
        assert_eq!(account.current_balance, 1000.0);
        assert_eq!(account.version, 1);
        assert!(account.is_current());
    }

    #[test]
    fn test_account_update_balance() {
        let bank_id = create_test_bank_id();
        let mut account = Account::new(
            "Test Checking".to_string(),
            "*1234".to_string(),
            bank_id,
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );

        account.update_balance(1500.0);
        assert_eq!(account.current_balance, 1500.0);
        assert_eq!(account.balance_change(), 500.0);
    }

    #[test]
    fn test_account_balance_checks() {
        let bank_id = create_test_bank_id();

        // Positive balance
        let positive_account = Account::new(
            "Positive".to_string(),
            "*1234".to_string(),
            bank_id.clone(),
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        assert!(positive_account.is_positive());
        assert!(!positive_account.is_overdrawn());

        // Negative balance (overdrawn)
        let mut overdrawn_account = Account::new(
            "Overdrawn".to_string(),
            "*5678".to_string(),
            bank_id,
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        overdrawn_account.update_balance(-50.0);
        assert!(!overdrawn_account.is_positive());
        assert!(overdrawn_account.is_overdrawn());
    }

    #[test]
    fn test_mask_account_number() {
        assert_eq!(Account::mask_account_number("1234567890"), "*7890");
        assert_eq!(Account::mask_account_number("1234"), "1234");
        assert_eq!(Account::mask_account_number("123"), "123");
        assert_eq!(Account::mask_account_number(""), "");
    }

    #[test]
    fn test_account_registry_register() {
        let mut registry = AccountRegistry::new();
        let bank_id = create_test_bank_id();

        let account = Account::new(
            "Test Checking".to_string(),
            "*1234".to_string(),
            bank_id,
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );

        registry.register(account);
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_account_registry_find_by_name() {
        let mut registry = AccountRegistry::new();
        let bank_id = create_test_bank_id();

        let account = Account::new(
            "Test Checking".to_string(),
            "*1234".to_string(),
            bank_id,
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        registry.register(account);

        // Find by exact name
        let found = registry.find_by_name("Test Checking");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Checking");

        // Case insensitive
        let found2 = registry.find_by_name("test checking");
        assert!(found2.is_some());

        // Unknown account
        let unknown = registry.find_by_name("Unknown Account");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_account_registry_find_by_id() {
        let mut registry = AccountRegistry::new();
        let bank_id = create_test_bank_id();

        let account = Account::new(
            "Test Checking".to_string(),
            "*1234".to_string(),
            bank_id,
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        let account_id = account.id.clone();
        registry.register(account);

        let found = registry.find_by_id(&account_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Checking");

        let not_found = registry.find_by_id("non-existent-uuid");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_account_registry_find_by_account_number() {
        let mut registry = AccountRegistry::new();
        let bank_id = create_test_bank_id();

        let account = Account::new(
            "Test Checking".to_string(),
            "*1234".to_string(),
            bank_id,
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        registry.register(account);

        let found = registry.find_by_account_number("*1234");
        assert!(found.is_some());
        assert_eq!(found.unwrap().account_number, "*1234");
    }

    #[test]
    fn test_account_registry_by_bank() {
        let mut registry = AccountRegistry::new();
        let bank_id = create_test_bank_id();
        let other_bank_id = create_test_bank_id();

        // Add 2 accounts for bank1
        let account1 = Account::new(
            "Checking *1234".to_string(),
            "*1234".to_string(),
            bank_id.clone(),
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        registry.register(account1);

        let account2 = Account::new(
            "Savings *5678".to_string(),
            "*5678".to_string(),
            bank_id.clone(),
            AccountType::Savings,
            "USD".to_string(),
            5000.0,
        );
        registry.register(account2);

        // Add 1 account for bank2
        let account3 = Account::new(
            "Other Checking".to_string(),
            "*9999".to_string(),
            other_bank_id.clone(),
            AccountType::Checking,
            "USD".to_string(),
            2000.0,
        );
        registry.register(account3);

        let bank1_accounts = registry.by_bank(&bank_id);
        assert_eq!(bank1_accounts.len(), 2);

        let bank2_accounts = registry.by_bank(&other_bank_id);
        assert_eq!(bank2_accounts.len(), 1);
    }

    #[test]
    fn test_account_registry_by_type() {
        let mut registry = AccountRegistry::new();
        let bank_id = create_test_bank_id();

        let checking = Account::new(
            "Checking".to_string(),
            "*1234".to_string(),
            bank_id.clone(),
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        registry.register(checking);

        let savings = Account::new(
            "Savings".to_string(),
            "*5678".to_string(),
            bank_id.clone(),
            AccountType::Savings,
            "USD".to_string(),
            5000.0,
        );
        registry.register(savings);

        let credit = Account::new(
            "Credit".to_string(),
            "*9999".to_string(),
            bank_id,
            AccountType::Credit,
            "USD".to_string(),
            -500.0,
        );
        registry.register(credit);

        let checking_accounts = registry.by_type(AccountType::Checking);
        assert_eq!(checking_accounts.len(), 1);

        let savings_accounts = registry.by_type(AccountType::Savings);
        assert_eq!(savings_accounts.len(), 1);

        let credit_accounts = registry.by_type(AccountType::Credit);
        assert_eq!(credit_accounts.len(), 1);
    }

    #[test]
    fn test_account_registry_by_currency() {
        let mut registry = AccountRegistry::new();
        let bank_id = create_test_bank_id();

        let usd_account = Account::new(
            "USD Account".to_string(),
            "*1234".to_string(),
            bank_id.clone(),
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        registry.register(usd_account);

        let mxn_account = Account::new(
            "MXN Account".to_string(),
            "*5678".to_string(),
            bank_id,
            AccountType::Checking,
            "MXN".to_string(),
            20000.0,
        );
        registry.register(mxn_account);

        let usd_accounts = registry.by_currency("USD");
        assert_eq!(usd_accounts.len(), 1);

        let mxn_accounts = registry.by_currency("MXN");
        assert_eq!(mxn_accounts.len(), 1);
    }

    #[test]
    fn test_account_registry_get_id() {
        let mut registry = AccountRegistry::new();
        let bank_id = create_test_bank_id();

        let account = Account::new(
            "Test Checking".to_string(),
            "*1234".to_string(),
            bank_id,
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        registry.register(account);

        // Get UUID for account name
        let account_id = registry.get_id("Test Checking");
        assert!(account_id.is_some());

        let account_id2 = registry.get_id("test checking"); // Case insensitive
        assert!(account_id2.is_some());

        // Same account should give same ID
        assert_eq!(account_id, account_id2);

        // Unknown account
        let unknown_id = registry.get_id("Unknown");
        assert!(unknown_id.is_none());
    }

    #[test]
    fn test_account_registry_total_balance() {
        let mut registry = AccountRegistry::new();
        let bank_id = create_test_bank_id();

        let account1 = Account::new(
            "Account 1".to_string(),
            "*1234".to_string(),
            bank_id.clone(),
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        registry.register(account1);

        let account2 = Account::new(
            "Account 2".to_string(),
            "*5678".to_string(),
            bank_id,
            AccountType::Savings,
            "USD".to_string(),
            5000.0,
        );
        registry.register(account2);

        let total = registry.total_balance();
        assert_eq!(total, 6000.0);
    }

    #[test]
    fn test_account_registry_total_balance_by_currency() {
        let mut registry = AccountRegistry::new();
        let bank_id = create_test_bank_id();

        let usd_account = Account::new(
            "USD Account".to_string(),
            "*1234".to_string(),
            bank_id.clone(),
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        registry.register(usd_account);

        let mxn_account = Account::new(
            "MXN Account".to_string(),
            "*5678".to_string(),
            bank_id,
            AccountType::Checking,
            "MXN".to_string(),
            20000.0,
        );
        registry.register(mxn_account);

        let usd_total = registry.total_balance_by_currency("USD");
        assert_eq!(usd_total, 1000.0);

        let mxn_total = registry.total_balance_by_currency("MXN");
        assert_eq!(mxn_total, 20000.0);
    }

    #[test]
    fn test_account_registry_positive_accounts() {
        let mut registry = AccountRegistry::new();
        let bank_id = create_test_bank_id();

        let mut positive = Account::new(
            "Positive".to_string(),
            "*1234".to_string(),
            bank_id.clone(),
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        registry.register(positive.clone());

        let mut negative = Account::new(
            "Negative".to_string(),
            "*5678".to_string(),
            bank_id,
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        negative.update_balance(-500.0);
        registry.register(negative);

        let positive_accounts = registry.positive_accounts();
        assert_eq!(positive_accounts.len(), 1);
        assert_eq!(positive_accounts[0].name, "Positive");
    }

    #[test]
    fn test_account_registry_overdrawn_accounts() {
        let mut registry = AccountRegistry::new();
        let bank_id = create_test_bank_id();

        let mut positive = Account::new(
            "Positive".to_string(),
            "*1234".to_string(),
            bank_id.clone(),
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        registry.register(positive);

        let mut overdrawn = Account::new(
            "Overdrawn".to_string(),
            "*5678".to_string(),
            bank_id,
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        overdrawn.update_balance(-500.0);
        registry.register(overdrawn);

        let overdrawn_accounts = registry.overdrawn_accounts();
        assert_eq!(overdrawn_accounts.len(), 1);
        assert_eq!(overdrawn_accounts[0].name, "Overdrawn");
    }

    #[test]
    fn test_account_versioning() {
        let bank_id = create_test_bank_id();
        let account = Account::new(
            "Test Account".to_string(),
            "*1234".to_string(),
            bank_id,
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );

        let original_version = account.version;
        let original_valid_from = account.valid_from;

        // Create next version
        let next = account.next_version();

        assert_eq!(next.version, original_version + 1);
        assert!(next.valid_from > original_valid_from);
        assert!(next.is_current());
        assert_eq!(next.id, account.id); // Identity remains the same!
    }

    // ========================================================================
    // BADGE 25: TEMPORAL PERSISTENCE TESTS
    // ========================================================================

    #[test]
    fn test_account_multi_version_storage() {
        let mut registry = AccountRegistry::new();
        let bank_id = create_test_bank_id();

        let account = Account::new(
            "Test Account".to_string(),
            "*1234".to_string(),
            bank_id,
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        let account_id = account.id.clone();
        registry.register(account);

        assert_eq!(registry.get_all_versions(&account_id).len(), 1);

        registry
            .update_account(&account_id, |a| {
                a.current_balance = 2000.0;
            })
            .unwrap();

        let versions = registry.get_all_versions(&account_id);
        assert_eq!(versions.len(), 2);

        assert!(versions[0].valid_until.is_some());
        assert_eq!(versions[0].version, 1);

        assert!(versions[1].valid_until.is_none());
        assert_eq!(versions[1].version, 2);
    }

    #[test]
    fn test_account_temporal_query() {
        use chrono::Duration;

        let mut registry = AccountRegistry::new();
        let bank_id = create_test_bank_id();

        let account = Account::new(
            "Test Account".to_string(),
            "*1234".to_string(),
            bank_id,
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        let account_id = account.id.clone();
        let t0 = Utc::now();

        registry.register(account);

        std::thread::sleep(std::time::Duration::from_millis(10));
        let t1 = Utc::now();

        registry
            .update_account(&account_id, |a| {
                a.current_balance = 2000.0;
            })
            .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));
        let t2 = Utc::now();

        let before = t0 - Duration::seconds(1);
        assert!(registry.get_account_at_time(&account_id, before).is_none());

        let at_t1 = registry.get_account_at_time(&account_id, t1).unwrap();
        assert_eq!(at_t1.version, 1);
        assert_eq!(at_t1.current_balance, 1000.0);

        let at_t2 = registry.get_account_at_time(&account_id, t2).unwrap();
        assert_eq!(at_t2.version, 2);
        assert_eq!(at_t2.current_balance, 2000.0);
    }

    #[test]
    fn test_account_update_preserves_history() {
        let mut registry = AccountRegistry::new();
        let bank_id = create_test_bank_id();

        let account = Account::new(
            "Test Account".to_string(),
            "*1234".to_string(),
            bank_id,
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        let account_id = account.id.clone();
        registry.register(account);

        let v1 = registry.get_current_version(&account_id).unwrap();
        assert_eq!(v1.current_balance, 1000.0);
        assert_eq!(v1.account_type, AccountType::Checking);

        registry
            .update_account(&account_id, |a| {
                a.current_balance = 2000.0;
            })
            .unwrap();

        let v2 = registry.get_current_version(&account_id).unwrap();
        assert_eq!(v2.current_balance, 2000.0);
        assert_eq!(v2.version, 2);

        registry
            .update_account(&account_id, |a| {
                a.account_type = AccountType::Savings;
            })
            .unwrap();

        let v3 = registry.get_current_version(&account_id).unwrap();
        assert_eq!(v3.current_balance, 2000.0);
        assert_eq!(v3.account_type, AccountType::Savings);
        assert_eq!(v3.version, 3);

        let all_versions = registry.get_all_versions(&account_id);
        assert_eq!(all_versions.len(), 3);

        assert_eq!(all_versions[0].current_balance, 1000.0);
        assert_eq!(all_versions[0].account_type, AccountType::Checking);

        assert_eq!(all_versions[1].current_balance, 2000.0);
        assert_eq!(all_versions[1].account_type, AccountType::Checking);

        assert_eq!(all_versions[2].current_balance, 2000.0);
        assert_eq!(all_versions[2].account_type, AccountType::Savings);
    }

    #[test]
    fn test_account_update_expires_previous_version() {
        let mut registry = AccountRegistry::new();
        let bank_id = create_test_bank_id();

        let account = Account::new(
            "Test Account".to_string(),
            "*1234".to_string(),
            bank_id,
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        let account_id = account.id.clone();
        registry.register(account);

        let v1_before = registry.get_current_version(&account_id).unwrap();
        assert!(v1_before.valid_until.is_none());

        registry
            .update_account(&account_id, |a| {
                a.current_balance = 2000.0;
            })
            .unwrap();

        let versions = registry.get_all_versions(&account_id);
        let v1_after = versions.iter().find(|a| a.version == 1).unwrap();
        assert!(v1_after.valid_until.is_some());

        let v2 = versions.iter().find(|a| a.version == 2).unwrap();
        assert!(v2.valid_until.is_none());
    }

    #[test]
    fn test_account_identity_persists_across_versions() {
        let mut registry = AccountRegistry::new();
        let bank_id = create_test_bank_id();

        let account = Account::new(
            "Test Account".to_string(),
            "*1234".to_string(),
            bank_id,
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        let account_id = account.id.clone();
        registry.register(account);

        for i in 0..5 {
            registry
                .update_account(&account_id, |a| {
                    a.current_balance = 1000.0 + (i as f64 * 100.0);
                })
                .unwrap();
        }

        let versions = registry.get_all_versions(&account_id);
        assert_eq!(versions.len(), 6);

        for version in versions {
            assert_eq!(version.id, account_id);
        }
    }

    #[test]
    fn test_account_get_current_version_returns_latest() {
        let mut registry = AccountRegistry::new();
        let bank_id = create_test_bank_id();

        let account = Account::new(
            "Test Account".to_string(),
            "*1234".to_string(),
            bank_id,
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        let account_id = account.id.clone();
        registry.register(account);

        for i in 1..=3 {
            registry
                .update_account(&account_id, |a| {
                    a.current_balance = 1000.0 + (i as f64 * 100.0);
                })
                .unwrap();
        }

        let current = registry.get_current_version(&account_id).unwrap();
        assert_eq!(current.version, 4);
        assert_eq!(current.current_balance, 1300.0);
        assert!(current.valid_until.is_none());
    }

    #[test]
    fn test_account_all_only_returns_current_versions() {
        let mut registry = AccountRegistry::new();
        let bank_id = create_test_bank_id();

        let account1 = Account::new(
            "Account 1".to_string(),
            "*1234".to_string(),
            bank_id.clone(),
            AccountType::Checking,
            "USD".to_string(),
            1000.0,
        );
        let account1_id = account1.id.clone();

        let account2 = Account::new(
            "Account 2".to_string(),
            "*5678".to_string(),
            bank_id,
            AccountType::Savings,
            "USD".to_string(),
            2000.0,
        );
        let account2_id = account2.id.clone();

        registry.register(account1);
        registry.register(account2);

        assert_eq!(registry.all_accounts().len(), 2);

        for i in 1..=3 {
            registry
                .update_account(&account1_id, |a| {
                    a.current_balance = 1000.0 + (i as f64 * 100.0);
                })
                .unwrap();
        }

        for i in 1..=2 {
            registry
                .update_account(&account2_id, |a| {
                    a.current_balance = 2000.0 + (i as f64 * 100.0);
                })
                .unwrap();
        }

        let test_account_versions: Vec<Account> = registry
            .versions
            .read()
            .unwrap()
            .iter()
            .filter(|a| a.id == account1_id || a.id == account2_id)
            .cloned()
            .collect();
        assert_eq!(test_account_versions.len(), 7);

        assert_eq!(registry.all_accounts().len(), 2);

        let all_accounts = registry.all_accounts();
        for account in all_accounts {
            assert!(account.is_current());
        }
    }

    #[test]
    fn test_account_update_nonexistent_fails() {
        let mut registry = AccountRegistry::new();

        let result = registry.update_account("non-existent-id", |a| {
            a.current_balance = 9999.0;
        });

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Account not found"));
    }
}
