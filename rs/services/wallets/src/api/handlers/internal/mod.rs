//! Internal API endpoints - only available with internal-api feature
//!
//! These endpoints are intended for cross-service communication
//! and should not be exposed in production environments without proper access controls.

pub mod entity_app_wallets;
pub mod entity_wallets;
pub mod openapi;
pub mod send_tx;
pub mod signing;

pub use openapi::InternalApiDoc;
