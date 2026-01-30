//! Internal API endpoints - only available with internal-api feature
//!
//! These endpoints are intended for administrative or debugging purposes
//! and should not be exposed in production environments without proper access controls.

pub mod entities;
pub mod openapi;

pub use entities::*;
pub use openapi::InternalApiDoc;
