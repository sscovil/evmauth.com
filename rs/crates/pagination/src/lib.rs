//! Cursor-based pagination following the Relay GraphQL Connections Specification.
//!
//! Provides types, traits, and query helpers for implementing cursor-based pagination
//! over PostgreSQL queries using SQLx.

mod error;
mod query;
mod types;

pub use error::PaginationError;
pub use query::{apply_cursor_pagination, reverse_if_backward};
pub use types::{Cursor, Page, PageDirection, Pageable, PaginatedResponse};

/// Re-export of the `with_pagination` derive macro from `pagination_macros`.
pub use pagination_macros::with_pagination;
