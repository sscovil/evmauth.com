use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::PaginationError;

/// Pagination direction for cursor-based pagination
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PageDirection {
    Forward,
    Backward,
}

impl Default for PageDirection {
    fn default() -> Self {
        Self::Forward
    }
}

/// Cursor for pagination, encoded as base64(json({id, created_at}))
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cursor {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl Cursor {
    /// Create a new cursor from an item's id and creation timestamp
    pub fn new(id: Uuid, created_at: DateTime<Utc>) -> Self {
        Self { id, created_at }
    }

    /// Encode cursor to base64 JSON string
    pub fn encode(&self) -> String {
        // Cursor contains only Uuid and DateTime<Utc>, both of which have
        // infallible Serialize impls, so this cannot fail in practice.
        let json = serde_json::to_string(self).expect("cursor serialization is infallible");
        STANDARD.encode(json.as_bytes())
    }

    /// Decode cursor from base64 JSON string
    pub fn decode(encoded: &str) -> Result<Self, PaginationError> {
        let bytes = STANDARD
            .decode(encoded.as_bytes())
            .map_err(|e| PaginationError::InvalidCursor(format!("base64 decode failed: {e}")))?;

        let json = String::from_utf8(bytes)
            .map_err(|e| PaginationError::InvalidCursor(format!("utf-8 decode failed: {e}")))?;

        serde_json::from_str(&json)
            .map_err(|e| PaginationError::InvalidCursor(format!("json parse failed: {e}")))
    }
}

/// Page configuration for cursor-based pagination following Relay spec
///
/// Use `first` and `after` for forward pagination, or `last` and `before` for backward pagination.
/// The Relay spec discourages using both `first` and `last` together.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Page {
    /// Number of items to return (forward pagination)
    pub first: Option<i64>,
    /// Cursor to paginate after (forward pagination)
    pub after: Option<String>,
    /// Number of items to return (backward pagination)
    pub last: Option<i64>,
    /// Cursor to paginate before (backward pagination)
    pub before: Option<String>,
}

impl Page {
    /// Create a default page configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate that pagination parameters follow Relay spec rules
    pub fn validate(&self) -> Result<(), PaginationError> {
        // Relay spec discourages using both first and last
        if self.first.is_some() && self.last.is_some() {
            return Err(PaginationError::InvalidParameters(
                "cannot specify both first and last".to_string(),
            ));
        }

        // Cannot use after with last/before (after is for forward pagination)
        if self.after.is_some() && (self.last.is_some() || self.before.is_some()) {
            return Err(PaginationError::InvalidParameters(
                "cannot use after with last or before".to_string(),
            ));
        }

        // Cannot use before with first/after (before is for backward pagination)
        if self.before.is_some() && self.first.is_some() {
            return Err(PaginationError::InvalidParameters(
                "cannot use before with first".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the decoded cursor (from either after or before)
    pub fn cursor(&self) -> Result<Option<Cursor>, PaginationError> {
        if let Some(after) = &self.after {
            Ok(Some(Cursor::decode(after)?))
        } else if let Some(before) = &self.before {
            Ok(Some(Cursor::decode(before)?))
        } else {
            Ok(None)
        }
    }

    /// Get the limit (from either first or last), clamped to 1-100
    pub fn limit(&self) -> i64 {
        let limit = self.first.or(self.last).unwrap_or(20);
        limit.clamp(1, 100)
    }

    /// Determine pagination direction based on which parameters are set
    pub fn direction(&self) -> PageDirection {
        if self.before.is_some() || self.last.is_some() {
            PageDirection::Backward
        } else {
            PageDirection::Forward
        }
    }
}

/// Trait for types that can be paginated
///
/// Types implementing this trait can provide cursor information (id and created_at)
/// needed for cursor-based pagination.
pub trait Pageable {
    /// The unique identifier for cursor positioning
    fn cursor_id(&self) -> Uuid;
    /// The creation timestamp for cursor positioning
    fn cursor_created_at(&self) -> DateTime<Utc>;
}

/// Standard paginated response wrapper following Relay spec
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    /// Cursor of the first item in results (for backward pagination)
    pub start_cursor: Option<String>,
    /// Cursor of the last item in results (for forward pagination)
    pub end_cursor: Option<String>,
    /// Whether more items exist in the forward direction
    pub has_next_page: bool,
    /// Whether more items exist in the backward direction
    pub has_previous_page: bool,
}

impl<T> PaginatedResponse<T> {
    /// Create a paginated response with explicit cursor metadata
    pub fn new(
        data: Vec<T>,
        start_cursor: Option<String>,
        end_cursor: Option<String>,
        has_next_page: bool,
        has_previous_page: bool,
    ) -> Self {
        Self {
            data,
            start_cursor,
            end_cursor,
            has_next_page,
            has_previous_page,
        }
    }

    /// Build a paginated response from fetched items following Relay spec
    ///
    /// This method expects `items` to contain up to `page.limit() + 1` items.
    /// It will automatically:
    /// - Determine if there are more results in each direction
    /// - Trim the results to `page.limit()`
    /// - Create start/end cursors from the first and last items
    ///
    /// # Relay Pagination Logic
    ///
    /// For forward pagination (`first`/`after`):
    /// - `has_next_page`: True if we fetched more than limit (more items forward)
    /// - `has_previous_page`: True if an `after` cursor was provided
    /// - `end_cursor`: Use this with `after` param to get next page
    ///
    /// For backward pagination (`last`/`before`):
    /// - `has_next_page`: True if a `before` cursor was provided
    /// - `has_previous_page`: True if we fetched more than limit (more items backward)
    /// - `start_cursor`: Use this with `before` param to get previous page
    pub fn from_page(mut items: Vec<T>, page: &Page) -> Self
    where
        T: Pageable,
    {
        let limit = page.limit() as usize;
        let direction = page.direction();
        let has_cursor = page.after.is_some() || page.before.is_some();

        // Check if we have more results than the limit
        let has_more_in_direction = items.len() > limit;

        // Trim to the actual limit
        if has_more_in_direction {
            match direction {
                PageDirection::Forward => {
                    // Forward: keep first N items
                    items.truncate(limit);
                }
                PageDirection::Backward => {
                    // Backward: keep last N items (remove from beginning)
                    items.drain(0..items.len() - limit);
                }
            }
        }

        // Create cursors from first and last items
        let start_cursor = if !items.is_empty() {
            let first = &items[0];
            Some(Cursor::new(first.cursor_id(), first.cursor_created_at()).encode())
        } else {
            None
        };

        let end_cursor = if !items.is_empty() {
            let last = &items[items.len() - 1];
            Some(Cursor::new(last.cursor_id(), last.cursor_created_at()).encode())
        } else {
            None
        };

        // Determine has_next_page and has_previous_page based on direction
        let (has_next_page, has_previous_page) = match direction {
            PageDirection::Forward => {
                // Forward: has_more means more items forward, has_cursor means can go back
                (has_more_in_direction, has_cursor)
            }
            PageDirection::Backward => {
                // Backward: has_more means more items backward, has_cursor means can go forward
                (has_cursor, has_more_in_direction)
            }
        };

        Self {
            data: items,
            start_cursor,
            end_cursor,
            has_next_page,
            has_previous_page,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_encode_decode() {
        let id = Uuid::new_v4();
        let created_at = Utc::now();
        let cursor = Cursor::new(id, created_at);

        let encoded = cursor.encode();
        let decoded = Cursor::decode(&encoded).expect("Failed to decode cursor");

        assert_eq!(decoded.id, cursor.id);
        assert_eq!(decoded.created_at, cursor.created_at);
    }

    #[test]
    fn test_invalid_cursor() {
        let result = Cursor::decode("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_page_defaults() {
        let page = Page::new();
        assert_eq!(page.limit(), 20);
        assert_eq!(page.direction(), PageDirection::Forward);
        assert!(page.cursor().unwrap().is_none());
    }

    #[test]
    fn test_page_limit_clamping() {
        let page = Page {
            first: Some(200),
            ..Default::default()
        };
        assert_eq!(page.limit(), 100); // Should clamp to max

        let page = Page {
            first: Some(-5),
            ..Default::default()
        };
        assert_eq!(page.limit(), 1); // Should clamp to min
    }

    #[test]
    fn test_page_validation() {
        // Valid forward pagination
        let page = Page {
            first: Some(10),
            after: Some("cursor".to_string()),
            ..Default::default()
        };
        assert!(page.validate().is_ok());

        // Valid backward pagination
        let page = Page {
            last: Some(10),
            before: Some("cursor".to_string()),
            ..Default::default()
        };
        assert!(page.validate().is_ok());

        // Invalid: both first and last
        let page = Page {
            first: Some(10),
            last: Some(10),
            ..Default::default()
        };
        assert!(page.validate().is_err());

        // Invalid: after with last
        let page = Page {
            after: Some("cursor".to_string()),
            last: Some(10),
            ..Default::default()
        };
        assert!(page.validate().is_err());

        // Invalid: before with first
        let page = Page {
            before: Some("cursor".to_string()),
            first: Some(10),
            ..Default::default()
        };
        assert!(page.validate().is_err());
    }

    #[test]
    fn test_page_direction() {
        // Forward pagination
        let page = Page {
            first: Some(10),
            ..Default::default()
        };
        assert_eq!(page.direction(), PageDirection::Forward);

        let page = Page {
            first: Some(10),
            after: Some("cursor".to_string()),
            ..Default::default()
        };
        assert_eq!(page.direction(), PageDirection::Forward);

        // Backward pagination
        let page = Page {
            last: Some(10),
            ..Default::default()
        };
        assert_eq!(page.direction(), PageDirection::Backward);

        let page = Page {
            last: Some(10),
            before: Some("cursor".to_string()),
            ..Default::default()
        };
        assert_eq!(page.direction(), PageDirection::Backward);

        let page = Page {
            before: Some("cursor".to_string()),
            ..Default::default()
        };
        assert_eq!(page.direction(), PageDirection::Backward);
    }

    // Mock type for testing PaginatedResponse
    #[derive(Debug, Clone)]
    struct TestItem {
        id: Uuid,
        created_at: DateTime<Utc>,
        name: String,
    }

    impl Pageable for TestItem {
        fn cursor_id(&self) -> Uuid {
            self.id
        }

        fn cursor_created_at(&self) -> DateTime<Utc> {
            self.created_at
        }
    }

    impl TestItem {
        fn new(name: &str, id: Uuid, created_at: DateTime<Utc>) -> Self {
            Self {
                id,
                created_at,
                name: name.to_string(),
            }
        }
    }

    #[test]
    fn test_paginated_response_forward_with_extra_items() {
        // Test forward pagination with LIMIT+1 items (should truncate first item)
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        let now = Utc::now();

        let items = vec![
            TestItem::new("Item1", id1, now),
            TestItem::new("Item2", id2, now),
            TestItem::new("Item3", id3, now), // Extra item for has_next_page detection
        ];

        let page = Page {
            first: Some(2),
            after: Some("dummy_cursor".to_string()),
            ..Default::default()
        };

        let response = PaginatedResponse::from_page(items, &page);

        // Should keep first 2 items
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].name, "Item1");
        assert_eq!(response.data[1].name, "Item2");

        // Should have next page (because we had 3 items for limit of 2)
        assert!(response.has_next_page);
        // Should have previous page (because we provided an 'after' cursor)
        assert!(response.has_previous_page);

        // Cursors should be from first and last items in result
        assert!(response.start_cursor.is_some());
        assert!(response.end_cursor.is_some());
    }

    #[test]
    fn test_paginated_response_backward_with_extra_items() {
        // Test backward pagination with LIMIT+1 items (should truncate and keep last items)
        // This is the regression test for the bug we fixed
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        let now = Utc::now();

        // Items are already reversed by reverse_if_backward before calling from_page
        let items = vec![
            TestItem::new("Item1", id1, now),
            TestItem::new("Item2", id2, now),
            TestItem::new("Item3", id3, now), // Extra item for has_previous_page detection
        ];

        let page = Page {
            last: Some(2),
            before: Some("dummy_cursor".to_string()),
            ..Default::default()
        };

        let response = PaginatedResponse::from_page(items, &page);

        // Should keep LAST 2 items (Item2 and Item3), NOT first 2
        assert_eq!(response.data.len(), 2);
        assert_eq!(
            response.data[0].name, "Item2",
            "Should keep Item2 (second item)"
        );
        assert_eq!(
            response.data[1].name, "Item3",
            "Should keep Item3 (third item)"
        );

        // Should have previous page (because we had 3 items for limit of 2)
        assert!(response.has_previous_page);
        // Should have next page (because we provided a 'before' cursor)
        assert!(response.has_next_page);

        // Cursors should be from first and last items in result
        assert!(response.start_cursor.is_some());
        assert!(response.end_cursor.is_some());
    }

    #[test]
    fn test_paginated_response_forward_without_extra_items() {
        // Test forward pagination without extra items
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let now = Utc::now();

        let items = vec![
            TestItem::new("Item1", id1, now),
            TestItem::new("Item2", id2, now),
        ];

        let page = Page {
            first: Some(2),
            ..Default::default()
        };

        let response = PaginatedResponse::from_page(items, &page);

        // Should keep all items
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].name, "Item1");
        assert_eq!(response.data[1].name, "Item2");

        // Should NOT have next page (no extra items)
        assert!(!response.has_next_page);
        // Should NOT have previous page (no cursor provided)
        assert!(!response.has_previous_page);
    }

    #[test]
    fn test_paginated_response_backward_without_extra_items() {
        // Test backward pagination without extra items
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let now = Utc::now();

        let items = vec![
            TestItem::new("Item1", id1, now),
            TestItem::new("Item2", id2, now),
        ];

        let page = Page {
            last: Some(2),
            ..Default::default()
        };

        let response = PaginatedResponse::from_page(items, &page);

        // Should keep all items
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].name, "Item1");
        assert_eq!(response.data[1].name, "Item2");

        // Should NOT have previous page (no extra items)
        assert!(!response.has_previous_page);
        // Should NOT have next page (no cursor provided)
        assert!(!response.has_next_page);
    }

    #[test]
    fn test_paginated_response_empty_results() {
        // Test with no results
        let items: Vec<TestItem> = vec![];

        let page = Page {
            first: Some(10),
            ..Default::default()
        };

        let response = PaginatedResponse::from_page(items, &page);

        assert_eq!(response.data.len(), 0);
        assert!(!response.has_next_page);
        assert!(!response.has_previous_page);
        assert!(response.start_cursor.is_none());
        assert!(response.end_cursor.is_none());
    }

    #[test]
    fn test_paginated_response_cursors() {
        // Test that cursors are correctly generated from first and last items
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        let now = Utc::now();

        let items = vec![
            TestItem::new("Item1", id1, now),
            TestItem::new("Item2", id2, now),
            TestItem::new("Item3", id3, now),
        ];

        let page = Page {
            first: Some(3),
            ..Default::default()
        };

        let response = PaginatedResponse::from_page(items, &page);

        // Decode cursors and verify they match first and last items
        let start_cursor = Cursor::decode(response.start_cursor.as_ref().unwrap()).unwrap();
        let end_cursor = Cursor::decode(response.end_cursor.as_ref().unwrap()).unwrap();

        assert_eq!(start_cursor.id, id1);
        assert_eq!(end_cursor.id, id3);
        assert_eq!(start_cursor.created_at, now);
        assert_eq!(end_cursor.created_at, now);
    }
}
