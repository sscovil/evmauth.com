//! Internal API endpoints - only available with internal-api feature
//!
//! These endpoints are intended for cross-service communication
//! and should not be exposed in production environments without proper access controls.

pub mod openapi;
pub mod org_wallets;
pub mod person_app_wallets;
pub mod person_sub_orgs;
pub mod signing;

pub use openapi::InternalApiDoc;
