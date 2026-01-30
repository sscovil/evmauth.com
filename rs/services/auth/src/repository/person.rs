use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::domain::Person;
use crate::dto::request::{CreatePerson, UpdatePerson};

use super::error::RepositoryError;
use super::filter::PersonFilter;
use super::pagination::Page;

#[async_trait]
pub trait PersonRepository: Send + Sync {
    async fn create(&self, person: CreatePerson) -> Result<Person, RepositoryError>;
    async fn get(&self, id: Uuid) -> Result<Option<Person>, RepositoryError>;
    async fn list(&self, filter: PersonFilter, page: Page) -> Result<Vec<Person>, RepositoryError>;
    async fn update(&self, id: Uuid, update: UpdatePerson) -> Result<Person, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}

pub struct PersonRepositoryImpl<'a> {
    pool: &'a PgPool,
}

impl<'a> PersonRepositoryImpl<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> PersonRepository for PersonRepositoryImpl<'a> {
    async fn create(&self, person: CreatePerson) -> Result<Person, RepositoryError> {
        let result = sqlx::query_as!(
            Person,
            r#"
            INSERT INTO auth.people (display_name, description, auth_provider_name, auth_provider_ref, primary_email)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, display_name, description, auth_provider_name, auth_provider_ref, primary_email, created_at, updated_at
            "#,
            person.display_name,
            person.description,
            person.auth_provider_name,
            person.auth_provider_ref,
            person.primary_email
        )
        .fetch_one(self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return RepositoryError::ConstraintViolation("Email already exists".to_string());
                }
            }
            RepositoryError::Database(e)
        })?;

        Ok(result)
    }

    async fn get(&self, id: Uuid) -> Result<Option<Person>, RepositoryError> {
        let person = sqlx::query_as!(
            Person,
            r#"
            SELECT id, display_name, description, auth_provider_name, auth_provider_ref, primary_email, created_at, updated_at
            FROM auth.people
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(person)
    }

    async fn list(&self, filter: PersonFilter, page: Page) -> Result<Vec<Person>, RepositoryError> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT id, display_name, description, auth_provider_name, auth_provider_ref, primary_email, created_at, updated_at FROM auth.people WHERE 1=1"
        );

        if let Some(email) = &filter.email {
            query_builder.push(" AND primary_email = ");
            query_builder.push_bind(email);
        }

        if let Some(provider) = &filter.auth_provider {
            query_builder.push(" AND auth_provider_name = ");
            query_builder.push_bind(provider);
        }

        let search_pattern = filter.search.as_ref().map(|s| format!("%{}%", s));
        if let Some(pattern) = &search_pattern {
            query_builder.push(" AND (display_name ILIKE ");
            query_builder.push_bind(pattern);
            query_builder.push(" OR primary_email ILIKE ");
            query_builder.push_bind(pattern);
            query_builder.push(")");
        }

        // Apply cursor-based pagination (Relay spec compliant)
        pagination::apply_cursor_pagination(&mut query_builder, &page, None, None)?;

        let mut people = query_builder
            .build_query_as::<Person>()
            .fetch_all(self.pool)
            .await?;

        // Reverse results if backward pagination to maintain consistent ordering
        pagination::reverse_if_backward(&mut people, &page);

        Ok(people)
    }

    async fn update(&self, id: Uuid, update: UpdatePerson) -> Result<Person, RepositoryError> {
        let current = self.get(id).await?.ok_or(RepositoryError::NotFound)?;

        let display_name = update.display_name.unwrap_or(current.display_name);
        let description = update.description.or(current.description);
        let primary_email = update.primary_email.unwrap_or(current.primary_email);

        let person = sqlx::query_as!(
            Person,
            r#"
            UPDATE auth.people
            SET display_name = $1, description = $2, primary_email = $3
            WHERE id = $4
            RETURNING id, display_name, description, auth_provider_name, auth_provider_ref, primary_email, created_at, updated_at
            "#,
            display_name,
            description,
            primary_email,
            id
        )
        .fetch_one(self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return RepositoryError::ConstraintViolation("Email already exists".to_string());
                }
            }
            RepositoryError::Database(e)
        })?;

        Ok(person)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM auth.people
            WHERE id = $1
            "#,
            id
        )
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }
}
