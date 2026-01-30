# Pagination Crate

A Rust library for implementing cursor-based pagination following the [Relay Cursor Specification](https://relay.dev/graphql/connections.htm).

## Overview

This crate provides reusable pagination helpers for building APIs with consistent, efficient cursor-based pagination. It works with SQLx query builders and integrates seamlessly with Axum handlers.

## Features

- **Relay Spec Compliant**: Uses `first`/`after` and `last`/`before` parameters
- **Cursor-Based**: Stateless pagination using opaque base64-encoded cursors
- **SQL Query Helpers**: Automatic query building with `apply_cursor_pagination()`
- **Response Builders**: Automatic `has_next_page`/`has_previous_page` detection
- **Proc Macro**: `#[with_pagination]` attribute for easy query parameter integration
- **Type-Safe**: Compile-time validation with Rust's type system

## Quick Start

### 1. Add to Dependencies

Your `Cargo.toml` should include:

```toml
[dependencies]
pagination = { path = "../../crates/pagination" }
```

For handler files, also add the macro:

```toml
[dependencies]
pagination = { path = "../../crates/pagination", features = ["macros"] }
```

### 2. Make Your Domain Model Pageable

Implement the `Pageable` trait on your domain model:

```rust
use pagination::Pageable;
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub struct MyEntity {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    // ... other fields
}

impl Pageable for MyEntity {
    fn cursor_id(&self) -> Uuid {
        self.id
    }

    fn cursor_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
```

### 3. Add Pagination to Your Query Struct

Use the `#[with_pagination]` macro to automatically add Relay parameters:

```rust
use pagination::with_pagination;
use serde::Deserialize;
use utoipa::IntoParams;

#[with_pagination]
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListQuery {
    pub search: Option<String>,
    // The macro adds: first, after, last, before
}
```

This expands to:

```rust
pub struct ListQuery {
    pub search: Option<String>,
    pub first: Option<i64>,
    pub after: Option<String>,
    pub last: Option<i64>,
    pub before: Option<String>,
}

impl ListQuery {
    pub fn to_page(&self) -> Result<Page, PaginationError> { ... }
}
```

### 4. Use in Your Repository

Apply cursor pagination to your SQL queries:

```rust
use pagination::{apply_cursor_pagination, reverse_if_backward, Page};
use sqlx::{PgPool, Postgres, QueryBuilder};

pub async fn list(&self, page: Page) -> Result<Vec<MyEntity>, Error> {
    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT id, created_at, name FROM my_entities WHERE 1=1"
    );

    // Add filters here...

    // Apply cursor-based pagination (Relay spec compliant)
    apply_cursor_pagination(&mut query_builder, &page, None, None)?;

    let mut entities = query_builder
        .build_query_as::<MyEntity>()
        .fetch_all(self.pool)
        .await?;

    // Reverse if backward pagination to maintain consistent ordering
    reverse_if_backward(&mut entities, &page);

    Ok(entities)
}
```

### 5. Build the Response in Your Handler

Use `PaginatedResponse::from_page()` to automatically calculate pagination metadata:

```rust
use axum::{extract::{Query, State}, Json};
use pagination::PaginatedResponse;

pub async fn list_handler(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<PaginatedResponse<MyEntityResponse>>, ApiError> {
    let page = query.to_page()?;

    let entities = repo.list(page.clone()).await?;

    let responses: Vec<MyEntityResponse> = entities
        .into_iter()
        .map(Into::into)
        .collect();

    let response = PaginatedResponse::from_page(responses, &page);

    Ok(Json(response))
}
```

## API Parameters

Clients use Relay-style parameters:

### Forward Pagination
```
GET /api/items?first=20&after=eyJpZCI6IjEyMyIsImNyZWF0ZWRfYXQiOiIyMDI0LTAxLTAxIn0=
```

- `first`: Number of items to return (1-100, default 20)
- `after`: Cursor to paginate after (base64-encoded)

### Backward Pagination
```
GET /api/items?last=20&before=eyJpZCI6IjEyMyIsImNyZWF0ZWRfYXQiOiIyMDI0LTAxLTAxIn0=
```

- `last`: Number of items to return (1-100, default 20)
- `before`: Cursor to paginate before (base64-encoded)

## Response Format

```json
{
  "data": [...],
  "start_cursor": "eyJpZCI6IjEyMyIsImNyZWF0ZWRfYXQiOiIyMDI0LTAxLTAxIn0=",
  "end_cursor": "eyJpZCI6IjQ1NiIsImNyZWF0ZWRfYXQiOiIyMDI0LTAxLTAyIn0=",
  "has_next_page": true,
  "has_previous_page": false
}
```

- `data`: The requested items
- `start_cursor`: Cursor of the first item (use with `before` to go backward)
- `end_cursor`: Cursor of the last item (use with `after` to go forward)
- `has_next_page`: Whether more items exist in the forward direction
- `has_previous_page`: Whether more items exist in the backward direction

## Advanced Usage

### Custom ID Column (Composite Keys)

For tables without a single `id` column:

```rust
// Table: orgs_people (org_id, member_id, ...)
apply_cursor_pagination(&mut query_builder, &page, Some("member_id"), None)?;
```

### Table Aliases

When using SQL table aliases:

```rust
let mut query = QueryBuilder::new(
    "SELECT e.id, e.created_at, e.name FROM entities e WHERE 1=1"
);

apply_cursor_pagination(&mut query, &page, Some("e.id"), Some("e.created_at"))?;
```

### Response DTOs

Also implement `Pageable` on your response DTOs:

```rust
use pagination::Pageable;

pub struct MyEntityResponse {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    // ... other fields
}

impl Pageable for MyEntityResponse {
    fn cursor_id(&self) -> Uuid {
        self.id
    }

    fn cursor_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
```

This allows `PaginatedResponse::from_page()` to work with DTOs directly.

## Database Requirements

For optimal performance, ensure your table has an index on the pagination columns:

```sql
-- For standard pagination (id + created_at)
CREATE INDEX idx_table_pagination ON your_table (created_at, id);

-- For custom columns
CREATE INDEX idx_table_pagination ON your_table (timestamp_col, id_col);
```

The index should match the column order used in `apply_cursor_pagination()`.

## How It Works

1. **Cursor Encoding**: Cursors are base64-encoded JSON: `{"id": "...", "created_at": "..."}`
2. **SQL Row Comparison**: Uses tuple comparison for efficient filtering: `(created_at, id) > ($1, $2)`
3. **LIMIT + 1 Pattern**: Fetches one extra item to detect if more pages exist
4. **Backward Reversal**: Queries backward pagination in DESC order, then reverses results to maintain consistent ordering

## Validation Rules

The crate validates Relay spec rules:

- Cannot use both `first` and `last`
- Cannot use `after` with `last` or `before`
- Cannot use `before` with `first` or `after`

Invalid parameters return a `PaginationError::InvalidParameters` error.

## Integration Checklist

When adding pagination to a new endpoint:

- [ ] Implement `Pageable` on domain model
- [ ] Implement `Pageable` on response DTO
- [ ] Add `#[with_pagination]` to query struct
- [ ] Use `apply_cursor_pagination()` in repository
- [ ] Use `reverse_if_backward()` after fetching
- [ ] Use `PaginatedResponse::from_page()` in handler
- [ ] Add database index on `(created_at, id)` or equivalent columns
- [ ] Test forward and backward pagination
