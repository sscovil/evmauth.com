use sqlx::{Postgres, QueryBuilder};

use crate::types::{Page, PageDirection};

/// Apply cursor-based pagination to a SQL query builder following the Relay spec
///
/// This function modifies the provided QueryBuilder to add:
/// - Cursor-based WHERE clauses for forward/backward pagination
/// - ORDER BY clauses based on pagination direction
/// - LIMIT clause (with +1 to detect if more results exist)
///
/// Follows the Relay GraphQL Cursor Connections Specification:
/// https://relay.dev/graphql/connections.htm
///
/// # Requirements
///
/// Your query must:
/// - Select from tables that have a timestamp column (timestamptz) and an ID column (UUID)
/// - Have an index on `(<timestamp_column>, <id_column>)` for optimal performance
///
/// # Pagination Behavior
///
/// - **Forward pagination**: Returns items AFTER the cursor in ascending order
/// - **Backward pagination**: Returns items BEFORE the cursor in the same ascending order
///   (note: query uses DESC order but results must be reversed after fetching)
///
/// # Example
///
/// ```rust,ignore
/// use pagination::{apply_cursor_pagination, Page, reverse_if_backward};
/// use sqlx::{Postgres, QueryBuilder};
///
/// let page = Page::new().with_limit(10);
///
/// // Simple table (defaults to "created_at" and "id")
/// let mut query = QueryBuilder::<Postgres>::new(
///     "SELECT * FROM users WHERE active = true"
/// );
/// apply_cursor_pagination(&mut query, &page, None, None);
/// // Generates: ORDER BY created_at ASC, id ASC
///
/// // Composite key (custom ID column)
/// let mut query = QueryBuilder::<Postgres>::new(
///     "SELECT * FROM org_members WHERE org_id = ?"
/// );
/// apply_cursor_pagination(&mut query, &page, Some("member_id"), None);
/// // Generates: ORDER BY created_at ASC, member_id ASC
///
/// // With table alias (qualified column names)
/// let mut query = QueryBuilder::<Postgres>::new(
///     "SELECT e.* FROM entities e WHERE 1=1"
/// );
/// apply_cursor_pagination(&mut query, &page, Some("e.id"), Some("e.created_at"));
/// // Generates: ORDER BY e.created_at ASC, e.id ASC
///
/// let mut results = query.build_query_as::<User>()
///     .fetch_all(&pool)
///     .await?;
///
/// // Important: reverse results if backward pagination was used
/// reverse_if_backward(&mut results, &page);
/// ```
///
/// # Performance Notes
///
/// For optimal performance, ensure your table has an index:
/// ```sql
/// CREATE INDEX idx_table_pagination ON your_table (created_at, id);
/// -- or for custom columns:
/// CREATE INDEX idx_table_pagination ON your_table (timestamp_col, id_col);
/// ```
pub fn apply_cursor_pagination(
    query_builder: &mut QueryBuilder<Postgres>,
    page: &Page,
    id_column: Option<&str>,
    timestamp_column: Option<&str>,
) -> Result<(), crate::error::PaginationError> {
    let id_col = id_column.unwrap_or("id");
    let ts_col = timestamp_column.unwrap_or("created_at");

    // Validate pagination parameters
    page.validate()?;

    let cursor = page.cursor()?;
    let direction = page.direction();
    let limit = page.limit();

    // Add cursor-based WHERE clause if cursor is provided
    if let Some(cursor) = &cursor {
        match direction {
            PageDirection::Forward => {
                // Forward: get items AFTER the cursor
                query_builder.push(" AND (");
                query_builder.push(ts_col);
                query_builder.push(", ");
                query_builder.push(id_col);
                query_builder.push(") > (");
                query_builder.push_bind(cursor.created_at);
                query_builder.push(", ");
                query_builder.push_bind(cursor.id);
                query_builder.push(")");
            }
            PageDirection::Backward => {
                // Backward: get items BEFORE the cursor
                query_builder.push(" AND (");
                query_builder.push(ts_col);
                query_builder.push(", ");
                query_builder.push(id_col);
                query_builder.push(") < (");
                query_builder.push_bind(cursor.created_at);
                query_builder.push(", ");
                query_builder.push_bind(cursor.id);
                query_builder.push(")");
            }
        }
    }

    // Add ORDER BY based on direction
    match direction {
        PageDirection::Forward => {
            // Forward: ascending order
            query_builder.push(" ORDER BY ");
            query_builder.push(ts_col);
            query_builder.push(" ASC, ");
            query_builder.push(id_col);
            query_builder.push(" ASC");
        }
        PageDirection::Backward => {
            // Backward: descending order to get items before cursor
            // (caller must reverse results to maintain consistent ordering)
            query_builder.push(" ORDER BY ");
            query_builder.push(ts_col);
            query_builder.push(" DESC, ");
            query_builder.push(id_col);
            query_builder.push(" DESC");
        }
    }

    // Add LIMIT + 1 to detect if more results exist
    // (the +1 item will be used to set has_next_page/has_previous_page and then removed)
    query_builder.push(" LIMIT ");
    query_builder.push_bind(limit + 1);

    Ok(())
}

/// Reverse results if backward pagination was used
///
/// When paginating backward (before cursor), the SQL query fetches items in descending
/// order to get the correct set of results. This function reverses those results back
/// to ascending order to maintain consistent ordering per the Relay spec.
///
/// This ensures that regardless of pagination direction, results are always presented
/// in the same order (oldest to newest based on created_at).
///
/// # Example
///
/// ```rust,ignore
/// use pagination::{Page, PageDirection, reverse_if_backward};
///
/// let page = Page::new().with_direction(PageDirection::Backward);
/// let mut results = fetch_results(&page).await?;
///
/// // Results from DB are in DESC order: [item3, item2, item1]
/// reverse_if_backward(&mut results, &page);
/// // Now in ASC order: [item1, item2, item3]
/// ```
pub fn reverse_if_backward<T>(items: &mut [T], page: &Page) {
    if page.direction() == PageDirection::Backward {
        items.reverse();
    }
}
