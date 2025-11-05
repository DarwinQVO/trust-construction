// Entity Models - Badge 21-24
// Following Rich Hickey's philosophy: "Identity persists, values change"
//
// Each entity has:
// - Stable identity (UUID) that NEVER changes
// - Timeline of immutable values (with temporal tracking from Badge 19)
// - Registry for normalization and lookups

pub mod bank;
pub mod merchant;
pub mod category;
pub mod account;

pub use bank::{Bank, BankType, BankRegistry};
pub use merchant::{Merchant, MerchantType, MerchantRegistry};
pub use category::{Category, CategoryType, CategoryRegistry};
pub use account::{Account, AccountType, AccountRegistry};
