// 🏷️ Category Entity - Hierarchical categories with stable identity
// Badge 23: Following Rich Hickey's philosophy
//
// "Category name is a VALUE (can change), Category UUID is IDENTITY (never changes)"
//
// Problem solved:
// - Hierarchical categories: "Food & Dining" → "Restaurants" → "Fast Food"
// - Category trees for analytics (aggregate at any level)
// - Renaming doesn't break historical transactions
// - UUID provides stable foreign key for transactions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

// ============================================================================
// CATEGORY TYPE
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CategoryType {
    /// Expense category (money going out)
    Expense,

    /// Income category (money coming in)
    Income,

    /// Transfer between accounts (neutral)
    Transfer,
}

impl CategoryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CategoryType::Expense => "Expense",
            CategoryType::Income => "Income",
            CategoryType::Transfer => "Transfer",
        }
    }
}

// ============================================================================
// CATEGORY ENTITY
// ============================================================================

/// Category Entity - Rich Hickey's Identity/Value separation + Hierarchy
///
/// Identity: UUID (never changes)
/// Values: name, parent_id, category_type, etc. (can change over time)
/// Hierarchy: parent_id creates tree structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    // ========================================================================
    // IDENTITY (Badge 19 - never changes)
    // ========================================================================
    /// Stable identity (UUID) - NEVER changes
    pub id: String,

    // ========================================================================
    // VALUES (can change over time)
    // ========================================================================
    /// Category name (e.g., "Restaurants", "Café", "Starbucks")
    pub name: String,

    /// Parent category UUID (for hierarchy)
    /// Example: "Café" has parent_id = "Restaurants" UUID
    /// Root categories have parent_id = None
    pub parent_id: Option<String>,

    /// Type of category (Expense, Income, Transfer)
    pub category_type: CategoryType,

    /// Optional icon for UI (e.g., "🍽️", "☕", "🚗")
    pub icon: Option<String>,

    /// Optional color for UI (e.g., "#FF5733")
    pub color: Option<String>,

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

impl Category {
    /// Create new category entity with UUID
    pub fn new(
        name: String,
        parent_id: Option<String>,
        category_type: CategoryType,
    ) -> Self {
        let now = Utc::now();

        Category {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            parent_id,
            category_type,
            icon: None,
            color: None,
            version: 1,
            system_time: now,
            valid_from: now,
            valid_until: None,
            metadata: serde_json::json!({}),
        }
    }

    /// Create category with icon and color
    pub fn with_display(
        name: String,
        parent_id: Option<String>,
        category_type: CategoryType,
        icon: Option<String>,
        color: Option<String>,
    ) -> Self {
        let mut category = Self::new(name, parent_id, category_type);
        category.icon = icon;
        category.color = color;
        category
    }

    /// Check if this is a root category (no parent)
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    /// Check if this is a leaf category (has parent)
    pub fn is_leaf(&self) -> bool {
        self.parent_id.is_some()
    }

    /// Check if this version is current
    pub fn is_current(&self) -> bool {
        self.valid_until.is_none()
    }

    /// Create next version (for updating values)
    pub fn next_version(&self) -> Category {
        let now = Utc::now();
        let mut next = self.clone();
        next.version += 1;
        next.valid_from = now;
        next.valid_until = None;
        next
    }
}

// ============================================================================
// CATEGORY REGISTRY
// ============================================================================

/// Registry of all known categories
///
/// Badge 25: Multi-version storage - stores ALL versions, never deletes
///
/// This is a singleton that holds all Category entities in memory.
/// Supports hierarchical queries (parent/children relationships).
/// In production, this would be backed by a database with compound key (id, version).
pub struct CategoryRegistry {
    /// ALL versions of all categories (append-only, never delete)
    versions: Arc<RwLock<Vec<Category>>>,
}

impl CategoryRegistry {
    /// Create new empty registry
    pub fn new() -> Self {
        CategoryRegistry {
            versions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create registry with default categories pre-loaded
    pub fn with_defaults() -> Self {
        let mut registry = CategoryRegistry::new();
        registry.register_default_categories();
        registry
    }

    /// Initialize with hierarchical category structure
    ///
    /// Structure:
    /// - Food & Dining (Expense)
    ///   - Restaurants
    ///     - Fast Food
    ///     - Café
    ///   - Groceries
    /// - Transportation (Expense)
    ///   - Gas & Fuel
    ///   - Uber/Lyft
    /// - Shopping (Expense)
    ///   - General
    ///   - Online Shopping
    /// - Income
    ///   - Salary
    ///   - Business Income
    /// - Transfer
    ///   - Account Transfer
    fn register_default_categories(&mut self) {
        // ====================================================================
        // EXPENSE CATEGORIES
        // ====================================================================

        // Level 1: Food & Dining
        let food_dining = Category::with_display(
            "Food & Dining".to_string(),
            None,
            CategoryType::Expense,
            Some("🍽️".to_string()),
            Some("#FF5733".to_string()),
        );
        let food_dining_id = food_dining.id.clone();
        self.register(food_dining);

        // Level 2: Restaurants (under Food & Dining)
        let restaurants = Category::with_display(
            "Restaurants".to_string(),
            Some(food_dining_id.clone()),
            CategoryType::Expense,
            Some("🍴".to_string()),
            Some("#FF6B4A".to_string()),
        );
        let restaurants_id = restaurants.id.clone();
        self.register(restaurants);

        // Level 3: Fast Food (under Restaurants)
        let fast_food = Category::with_display(
            "Fast Food".to_string(),
            Some(restaurants_id.clone()),
            CategoryType::Expense,
            Some("🍔".to_string()),
            Some("#FF8C61".to_string()),
        );
        self.register(fast_food);

        // Level 3: Café (under Restaurants)
        let cafe = Category::with_display(
            "Café".to_string(),
            Some(restaurants_id),
            CategoryType::Expense,
            Some("☕".to_string()),
            Some("#8B4513".to_string()),
        );
        self.register(cafe);

        // Level 2: Groceries (under Food & Dining)
        let groceries = Category::with_display(
            "Groceries".to_string(),
            Some(food_dining_id),
            CategoryType::Expense,
            Some("🛒".to_string()),
            Some("#4CAF50".to_string()),
        );
        self.register(groceries);

        // Level 1: Transportation
        let transportation = Category::with_display(
            "Transportation".to_string(),
            None,
            CategoryType::Expense,
            Some("🚗".to_string()),
            Some("#2196F3".to_string()),
        );
        let transportation_id = transportation.id.clone();
        self.register(transportation);

        // Level 2: Gas & Fuel (under Transportation)
        let gas_fuel = Category::with_display(
            "Gas & Fuel".to_string(),
            Some(transportation_id.clone()),
            CategoryType::Expense,
            Some("⛽".to_string()),
            Some("#3F51B5".to_string()),
        );
        self.register(gas_fuel);

        // Level 2: Uber/Lyft (under Transportation)
        let rideshare = Category::with_display(
            "Uber/Lyft".to_string(),
            Some(transportation_id),
            CategoryType::Expense,
            Some("🚕".to_string()),
            Some("#03A9F4".to_string()),
        );
        self.register(rideshare);

        // Level 1: Shopping
        let shopping = Category::with_display(
            "Shopping".to_string(),
            None,
            CategoryType::Expense,
            Some("🛍️".to_string()),
            Some("#E91E63".to_string()),
        );
        let shopping_id = shopping.id.clone();
        self.register(shopping);

        // Level 2: General (under Shopping)
        let general_shopping = Category::with_display(
            "General".to_string(),
            Some(shopping_id.clone()),
            CategoryType::Expense,
            Some("🏪".to_string()),
            Some("#F06292".to_string()),
        );
        self.register(general_shopping);

        // Level 2: Online Shopping (under Shopping)
        let online_shopping = Category::with_display(
            "Online Shopping".to_string(),
            Some(shopping_id),
            CategoryType::Expense,
            Some("📦".to_string()),
            Some("#EC407A".to_string()),
        );
        self.register(online_shopping);

        // ====================================================================
        // INCOME CATEGORIES
        // ====================================================================

        // Level 1: Income
        let income = Category::with_display(
            "Income".to_string(),
            None,
            CategoryType::Income,
            Some("💰".to_string()),
            Some("#4CAF50".to_string()),
        );
        let income_id = income.id.clone();
        self.register(income);

        // Level 2: Salary (under Income)
        let salary = Category::with_display(
            "Salary".to_string(),
            Some(income_id.clone()),
            CategoryType::Income,
            Some("💼".to_string()),
            Some("#66BB6A".to_string()),
        );
        self.register(salary);

        // Level 2: Business Income (under Income)
        let business_income = Category::with_display(
            "Business Income".to_string(),
            Some(income_id),
            CategoryType::Income,
            Some("📈".to_string()),
            Some("#81C784".to_string()),
        );
        self.register(business_income);

        // ====================================================================
        // TRANSFER CATEGORIES
        // ====================================================================

        // Level 1: Transfer
        let transfer = Category::with_display(
            "Transfer".to_string(),
            None,
            CategoryType::Transfer,
            Some("🔄".to_string()),
            Some("#9E9E9E".to_string()),
        );
        let transfer_id = transfer.id.clone();
        self.register(transfer);

        // Level 2: Account Transfer (under Transfer)
        let account_transfer = Category::with_display(
            "Account Transfer".to_string(),
            Some(transfer_id),
            CategoryType::Transfer,
            Some("💸".to_string()),
            Some("#BDBDBD".to_string()),
        );
        self.register(account_transfer);
    }

    /// Register a new category version (append-only, never overwrites)
    pub fn register(&mut self, category: Category) {
        let mut versions = self.versions.write().unwrap();
        versions.push(category);
    }

    /// Get ALL versions of a category by ID
    pub fn get_all_versions(&self, id: &str) -> Vec<Category> {
        let versions = self.versions.read().unwrap();
        versions
            .iter()
            .filter(|c| c.id == id)
            .cloned()
            .collect()
    }

    /// Get current version of a category by ID
    pub fn get_current_version(&self, id: &str) -> Option<Category> {
        let versions = self.versions.read().unwrap();
        versions
            .iter()
            .filter(|c| c.id == id && c.is_current())
            .cloned()
            .next()
    }

    /// Get category as of a specific time (temporal query)
    pub fn get_category_at_time(&self, id: &str, as_of: DateTime<Utc>) -> Option<Category> {
        let versions = self.versions.read().unwrap();
        versions
            .iter()
            .filter(|c| c.id == id)
            .find(|c| {
                c.valid_from <= as_of
                    && (c.valid_until.is_none() || c.valid_until.unwrap() > as_of)
            })
            .cloned()
    }

    /// Update category (creates new version, expires old version)
    pub fn update_category<F>(&mut self, id: &str, mut update_fn: F) -> Result<(), String>
    where
        F: FnMut(&mut Category),
    {
        let now = Utc::now();

        let current = self
            .get_current_version(id)
            .ok_or_else(|| format!("Category not found: {}", id))?;

        let mut expired = current.clone();
        expired.valid_until = Some(now);

        let mut next = current.next_version();
        update_fn(&mut next);

        {
            let mut versions = self.versions.write().unwrap();
            versions.retain(|c| !(c.id == id && c.is_current()));
            versions.push(expired);
            versions.push(next);
        }

        Ok(())
    }

    /// Find category by name (exact match, case-insensitive) - returns current version
    pub fn find_by_name(&self, name: &str) -> Option<Category> {
        let versions = self.versions.read().unwrap();
        let lower_name = name.to_lowercase();
        versions
            .iter()
            .filter(|c| c.is_current())
            .find(|cat| cat.name.to_lowercase() == lower_name)
            .cloned()
    }

    /// Find category by UUID - returns current version
    pub fn find_by_id(&self, id: &str) -> Option<Category> {
        self.get_current_version(id)
    }

    /// Get all categories (current versions only)
    pub fn all_categories(&self) -> Vec<Category> {
        let versions = self.versions.read().unwrap();
        let mut current: Vec<Category> = versions.iter().filter(|c| c.is_current()).cloned().collect();

        current.sort_by(|a, b| a.id.cmp(&b.id).then(b.version.cmp(&a.version)));
        current.dedup_by(|a, b| a.id == b.id);

        current
    }

    /// Count total categories (current versions only)
    pub fn count(&self) -> usize {
        self.all_categories().len()
    }

    /// Get root categories (no parent, current versions only)
    pub fn root_categories(&self) -> Vec<Category> {
        self.all_categories().into_iter().filter(|cat| cat.is_root()).collect()
    }

    /// Get children of a category (current versions only)
    pub fn get_children(&self, parent_id: &str) -> Vec<Category> {
        self.all_categories()
            .into_iter()
            .filter(|cat| cat.parent_id.as_deref() == Some(parent_id))
            .collect()
    }

    /// Get parent of a category
    pub fn get_parent(&self, category: &Category) -> Option<Category> {
        category.parent_id.as_ref().and_then(|parent_id| self.find_by_id(parent_id))
    }

    /// Get full path of a category (root → ... → leaf)
    ///
    /// Example: "Fast Food" → ["Food & Dining", "Restaurants", "Fast Food"]
    pub fn get_path(&self, category: &Category) -> Vec<String> {
        let mut path = vec![category.name.clone()];
        let mut current = category.clone();

        while let Some(parent) = self.get_parent(&current) {
            path.insert(0, parent.name.clone());
            current = parent;
        }

        path
    }

    /// Get full path as string (root → ... → leaf)
    ///
    /// Example: "Fast Food" → "Food & Dining → Restaurants → Fast Food"
    pub fn get_path_string(&self, category: &Category) -> String {
        self.get_path(category).join(" → ")
    }

    /// Get categories by type (current versions only)
    pub fn by_type(&self, category_type: CategoryType) -> Vec<Category> {
        self.all_categories()
            .into_iter()
            .filter(|cat| cat.category_type == category_type)
            .collect()
    }

    /// Get category ID for a category name (for foreign key references)
    pub fn get_id(&self, name: &str) -> Option<String> {
        self.find_by_name(name).map(|cat| cat.id)
    }

    /// Check if category is an ancestor of another category
    ///
    /// Example: "Food & Dining" is ancestor of "Fast Food"
    pub fn is_ancestor(&self, ancestor_id: &str, descendant_id: &str) -> bool {
        if ancestor_id == descendant_id {
            return true;
        }

        let Some(descendant) = self.find_by_id(descendant_id) else {
            return false;
        };

        let Some(parent_id) = descendant.parent_id else {
            return false;
        };

        if parent_id == ancestor_id {
            return true;
        }

        self.is_ancestor(ancestor_id, &parent_id)
    }

    /// Get all descendants of a category (recursive)
    ///
    /// Example: "Food & Dining" → ["Restaurants", "Fast Food", "Café", "Groceries"]
    pub fn get_descendants(&self, category_id: &str) -> Vec<Category> {
        let mut descendants = Vec::new();
        let children = self.get_children(category_id);

        for child in children {
            descendants.push(child.clone());
            descendants.extend(self.get_descendants(&child.id));
        }

        descendants
    }

    /// Get category tree depth
    pub fn get_depth(&self, category: &Category) -> usize {
        let mut depth = 0;
        let mut current = category.clone();

        while let Some(parent) = self.get_parent(&current) {
            depth += 1;
            current = parent;
        }

        depth
    }
}

impl Default for CategoryRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_creation() {
        let category = Category::new(
            "Test Category".to_string(),
            None,
            CategoryType::Expense,
        );

        assert!(!category.id.is_empty());
        assert_eq!(category.name, "Test Category");
        assert_eq!(category.parent_id, None);
        assert_eq!(category.category_type, CategoryType::Expense);
        assert_eq!(category.version, 1);
        assert!(category.is_current());
        assert!(category.is_root());
    }

    #[test]
    fn test_category_with_parent() {
        let parent_id = uuid::Uuid::new_v4().to_string();
        let category = Category::new(
            "Child Category".to_string(),
            Some(parent_id.clone()),
            CategoryType::Expense,
        );

        assert_eq!(category.parent_id, Some(parent_id));
        assert!(!category.is_root());
        assert!(category.is_leaf());
    }

    #[test]
    fn test_category_with_display() {
        let category = Category::with_display(
            "Test".to_string(),
            None,
            CategoryType::Expense,
            Some("🍕".to_string()),
            Some("#FF5733".to_string()),
        );

        assert_eq!(category.icon, Some("🍕".to_string()));
        assert_eq!(category.color, Some("#FF5733".to_string()));
    }

    #[test]
    fn test_category_registry_initialization() {
        let registry = CategoryRegistry::with_defaults();

        // Should have 16 default categories (3 levels)
        assert_eq!(registry.count(), 16);

        let categories = registry.all_categories();
        let category_names: Vec<String> = categories.iter().map(|c| c.name.clone()).collect();

        // Level 1 (roots)
        assert!(category_names.contains(&"Food & Dining".to_string()));
        assert!(category_names.contains(&"Transportation".to_string()));
        assert!(category_names.contains(&"Shopping".to_string()));
        assert!(category_names.contains(&"Income".to_string()));
        assert!(category_names.contains(&"Transfer".to_string()));

        // Level 2
        assert!(category_names.contains(&"Restaurants".to_string()));
        assert!(category_names.contains(&"Groceries".to_string()));

        // Level 3
        assert!(category_names.contains(&"Fast Food".to_string()));
        assert!(category_names.contains(&"Café".to_string()));
    }

    #[test]
    fn test_category_registry_find_by_name() {
        let registry = CategoryRegistry::with_defaults();

        // Find by exact name
        let cafe = registry.find_by_name("Café");
        assert!(cafe.is_some());
        assert_eq!(cafe.unwrap().name, "Café");

        // Case insensitive
        let cafe2 = registry.find_by_name("café");
        assert!(cafe2.is_some());

        // Unknown category
        let unknown = registry.find_by_name("Unknown Category");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_category_registry_find_by_id() {
        let registry = CategoryRegistry::with_defaults();

        let cafe = registry.find_by_name("Café").unwrap();
        let cafe_id = cafe.id.clone();

        let found = registry.find_by_id(&cafe_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Café");

        let not_found = registry.find_by_id("non-existent-uuid");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_category_registry_root_categories() {
        let registry = CategoryRegistry::with_defaults();

        let roots = registry.root_categories();
        assert_eq!(roots.len(), 5); // Food & Dining, Transportation, Shopping, Income, Transfer

        let root_names: Vec<String> = roots.iter().map(|c| c.name.clone()).collect();
        assert!(root_names.contains(&"Food & Dining".to_string()));
        assert!(root_names.contains(&"Transportation".to_string()));
        assert!(root_names.contains(&"Shopping".to_string()));
        assert!(root_names.contains(&"Income".to_string()));
        assert!(root_names.contains(&"Transfer".to_string()));
    }

    #[test]
    fn test_category_registry_get_children() {
        let registry = CategoryRegistry::with_defaults();

        let food_dining = registry.find_by_name("Food & Dining").unwrap();
        let children = registry.get_children(&food_dining.id);

        assert_eq!(children.len(), 2); // Restaurants, Groceries
        let child_names: Vec<String> = children.iter().map(|c| c.name.clone()).collect();
        assert!(child_names.contains(&"Restaurants".to_string()));
        assert!(child_names.contains(&"Groceries".to_string()));
    }

    #[test]
    fn test_category_registry_get_parent() {
        let registry = CategoryRegistry::with_defaults();

        let cafe = registry.find_by_name("Café").unwrap();
        let parent = registry.get_parent(&cafe);

        assert!(parent.is_some());
        assert_eq!(parent.unwrap().name, "Restaurants");
    }

    #[test]
    fn test_category_registry_get_path() {
        let registry = CategoryRegistry::with_defaults();

        let fast_food = registry.find_by_name("Fast Food").unwrap();
        let path = registry.get_path(&fast_food);

        assert_eq!(path.len(), 3);
        assert_eq!(path[0], "Food & Dining");
        assert_eq!(path[1], "Restaurants");
        assert_eq!(path[2], "Fast Food");
    }

    #[test]
    fn test_category_registry_get_path_string() {
        let registry = CategoryRegistry::with_defaults();

        let cafe = registry.find_by_name("Café").unwrap();
        let path_string = registry.get_path_string(&cafe);

        assert_eq!(path_string, "Food & Dining → Restaurants → Café");
    }

    #[test]
    fn test_category_registry_by_type() {
        let registry = CategoryRegistry::with_defaults();

        let expenses = registry.by_type(CategoryType::Expense);
        assert!(expenses.len() > 0);

        let income = registry.by_type(CategoryType::Income);
        assert_eq!(income.len(), 3); // Income, Salary, Business Income

        let transfers = registry.by_type(CategoryType::Transfer);
        assert_eq!(transfers.len(), 2); // Transfer, Account Transfer
    }

    #[test]
    fn test_category_registry_get_id() {
        let registry = CategoryRegistry::with_defaults();

        // Get UUID for category name
        let cafe_id = registry.get_id("Café");
        assert!(cafe_id.is_some());

        let cafe_id2 = registry.get_id("café"); // Case insensitive
        assert!(cafe_id2.is_some());

        // Same category should give same ID
        assert_eq!(cafe_id, cafe_id2);

        // Unknown category
        let unknown_id = registry.get_id("Unknown");
        assert!(unknown_id.is_none());
    }

    #[test]
    fn test_category_registry_is_ancestor() {
        let registry = CategoryRegistry::with_defaults();

        let food_dining = registry.find_by_name("Food & Dining").unwrap();
        let fast_food = registry.find_by_name("Fast Food").unwrap();

        // Food & Dining is ancestor of Fast Food
        assert!(registry.is_ancestor(&food_dining.id, &fast_food.id));

        // Fast Food is NOT ancestor of Food & Dining
        assert!(!registry.is_ancestor(&fast_food.id, &food_dining.id));

        // Category is ancestor of itself
        assert!(registry.is_ancestor(&food_dining.id, &food_dining.id));
    }

    #[test]
    fn test_category_registry_get_descendants() {
        let registry = CategoryRegistry::with_defaults();

        let food_dining = registry.find_by_name("Food & Dining").unwrap();
        let descendants = registry.get_descendants(&food_dining.id);

        // Should include: Restaurants, Fast Food, Café, Groceries
        assert_eq!(descendants.len(), 4);

        let descendant_names: Vec<String> = descendants.iter().map(|c| c.name.clone()).collect();
        assert!(descendant_names.contains(&"Restaurants".to_string()));
        assert!(descendant_names.contains(&"Fast Food".to_string()));
        assert!(descendant_names.contains(&"Café".to_string()));
        assert!(descendant_names.contains(&"Groceries".to_string()));
    }

    #[test]
    fn test_category_registry_get_depth() {
        let registry = CategoryRegistry::with_defaults();

        let food_dining = registry.find_by_name("Food & Dining").unwrap();
        assert_eq!(registry.get_depth(&food_dining), 0); // Root

        let restaurants = registry.find_by_name("Restaurants").unwrap();
        assert_eq!(registry.get_depth(&restaurants), 1); // Level 2

        let fast_food = registry.find_by_name("Fast Food").unwrap();
        assert_eq!(registry.get_depth(&fast_food), 2); // Level 3
    }

    #[test]
    fn test_category_versioning() {
        let category = Category::new(
            "Test Category".to_string(),
            None,
            CategoryType::Expense,
        );

        let original_version = category.version;
        let original_valid_from = category.valid_from;

        // Create next version
        let next = category.next_version();

        assert_eq!(next.version, original_version + 1);
        assert!(next.valid_from > original_valid_from);
        assert!(next.is_current());
        assert_eq!(next.id, category.id); // Identity remains the same!
    }

    // ========================================================================
    // BADGE 25: TEMPORAL PERSISTENCE TESTS
    // ========================================================================

    #[test]
    fn test_category_multi_version_storage() {
        let mut registry = CategoryRegistry::new();

        let category = Category::new("Test Category".to_string(), None, CategoryType::Expense);
        let category_id = category.id.clone();
        registry.register(category);

        assert_eq!(registry.get_all_versions(&category_id).len(), 1);

        registry
            .update_category(&category_id, |c| {
                c.category_type = CategoryType::Income;
            })
            .unwrap();

        let versions = registry.get_all_versions(&category_id);
        assert_eq!(versions.len(), 2);

        assert!(versions[0].valid_until.is_some());
        assert_eq!(versions[0].version, 1);

        assert!(versions[1].valid_until.is_none());
        assert_eq!(versions[1].version, 2);
    }

    #[test]
    fn test_category_temporal_query() {
        use chrono::Duration;

        let mut registry = CategoryRegistry::new();

        let category = Category::new("Test Category".to_string(), None, CategoryType::Expense);
        let category_id = category.id.clone();
        let t0 = Utc::now();

        registry.register(category);

        std::thread::sleep(std::time::Duration::from_millis(10));
        let t1 = Utc::now();

        registry
            .update_category(&category_id, |c| {
                c.icon = Some("💰".to_string());
            })
            .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));
        let t2 = Utc::now();

        let before = t0 - Duration::seconds(1);
        assert!(registry.get_category_at_time(&category_id, before).is_none());

        let at_t1 = registry.get_category_at_time(&category_id, t1).unwrap();
        assert_eq!(at_t1.version, 1);
        assert!(at_t1.icon.is_none());

        let at_t2 = registry.get_category_at_time(&category_id, t2).unwrap();
        assert_eq!(at_t2.version, 2);
        assert_eq!(at_t2.icon, Some("💰".to_string()));
    }

    #[test]
    fn test_category_update_preserves_history() {
        let mut registry = CategoryRegistry::new();

        let category = Category::new("Test Category".to_string(), None, CategoryType::Expense);
        let category_id = category.id.clone();
        registry.register(category);

        let v1 = registry.get_current_version(&category_id).unwrap();
        assert_eq!(v1.category_type, CategoryType::Expense);
        assert!(v1.icon.is_none());

        registry
            .update_category(&category_id, |c| {
                c.category_type = CategoryType::Income;
            })
            .unwrap();

        let v2 = registry.get_current_version(&category_id).unwrap();
        assert_eq!(v2.category_type, CategoryType::Income);
        assert_eq!(v2.version, 2);

        registry
            .update_category(&category_id, |c| {
                c.icon = Some("💰".to_string());
            })
            .unwrap();

        let v3 = registry.get_current_version(&category_id).unwrap();
        assert_eq!(v3.category_type, CategoryType::Income);
        assert_eq!(v3.icon, Some("💰".to_string()));
        assert_eq!(v3.version, 3);

        let all_versions = registry.get_all_versions(&category_id);
        assert_eq!(all_versions.len(), 3);

        assert_eq!(all_versions[0].category_type, CategoryType::Expense);
        assert!(all_versions[0].icon.is_none());

        assert_eq!(all_versions[1].category_type, CategoryType::Income);
        assert!(all_versions[1].icon.is_none());

        assert_eq!(all_versions[2].category_type, CategoryType::Income);
        assert_eq!(all_versions[2].icon, Some("💰".to_string()));
    }

    #[test]
    fn test_category_update_expires_previous_version() {
        let mut registry = CategoryRegistry::new();

        let category = Category::new("Test Category".to_string(), None, CategoryType::Expense);
        let category_id = category.id.clone();
        registry.register(category);

        let v1_before = registry.get_current_version(&category_id).unwrap();
        assert!(v1_before.valid_until.is_none());

        registry
            .update_category(&category_id, |c| {
                c.category_type = CategoryType::Income;
            })
            .unwrap();

        let versions = registry.get_all_versions(&category_id);
        let v1_after = versions.iter().find(|c| c.version == 1).unwrap();
        assert!(v1_after.valid_until.is_some());

        let v2 = versions.iter().find(|c| c.version == 2).unwrap();
        assert!(v2.valid_until.is_none());
    }

    #[test]
    fn test_category_identity_persists_across_versions() {
        let mut registry = CategoryRegistry::new();

        let category = Category::new("Test Category".to_string(), None, CategoryType::Expense);
        let category_id = category.id.clone();
        registry.register(category);

        for i in 0..5 {
            registry
                .update_category(&category_id, |c| {
                    c.name = format!("Category {}", i);
                })
                .unwrap();
        }

        let versions = registry.get_all_versions(&category_id);
        assert_eq!(versions.len(), 6);

        for version in versions {
            assert_eq!(version.id, category_id);
        }
    }

    #[test]
    fn test_category_get_current_version_returns_latest() {
        let mut registry = CategoryRegistry::new();

        let category = Category::new("Test Category".to_string(), None, CategoryType::Expense);
        let category_id = category.id.clone();
        registry.register(category);

        for i in 1..=3 {
            registry
                .update_category(&category_id, |c| {
                    c.name = format!("V{}", i);
                })
                .unwrap();
        }

        let current = registry.get_current_version(&category_id).unwrap();
        assert_eq!(current.version, 4);
        assert_eq!(current.name, "V3");
        assert!(current.valid_until.is_none());
    }

    #[test]
    fn test_category_all_only_returns_current_versions() {
        let mut registry = CategoryRegistry::with_defaults();

        let category1 = Category::new("Category 1".to_string(), None, CategoryType::Expense);
        let category1_id = category1.id.clone();
        let category2 = Category::new("Category 2".to_string(), None, CategoryType::Income);
        let category2_id = category2.id.clone();

        registry.register(category1);
        registry.register(category2);

        let initial_count = registry.all_categories().len();
        assert_eq!(initial_count, 18); // 16 default + 2 new

        for i in 1..=3 {
            registry
                .update_category(&category1_id, |c| {
                    c.name = format!("V{}", i);
                })
                .unwrap();
        }

        for i in 1..=2 {
            registry
                .update_category(&category2_id, |c| {
                    c.name = format!("V{}", i);
                })
                .unwrap();
        }

        let test_category_versions: Vec<Category> = registry
            .versions
            .read()
            .unwrap()
            .iter()
            .filter(|c| c.id == category1_id || c.id == category2_id)
            .cloned()
            .collect();
        assert_eq!(test_category_versions.len(), 7);

        assert_eq!(registry.all_categories().len(), 18);

        let all_categories = registry.all_categories();
        for category in all_categories {
            assert!(category.is_current());
        }
    }

    #[test]
    fn test_category_update_nonexistent_fails() {
        let mut registry = CategoryRegistry::new();

        let result = registry.update_category("non-existent-id", |c| {
            c.name = "XX".to_string();
        });

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Category not found"));
    }
}
