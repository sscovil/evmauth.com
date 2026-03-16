pub mod entity_app_wallets;
pub mod entity_wallets;
pub mod health;

// Conditionally compile internal module
#[cfg(feature = "internal-api")]
pub mod internal;
