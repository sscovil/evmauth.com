//! Internal API endpoints - only available with internal-api feature
//!
//! These endpoints are intended for cross-service communication
//! and should not be exposed in production environments without proper access controls.

pub mod app_registrations;
pub mod contracts;
pub mod openapi;

pub use openapi::InternalApiDoc;
