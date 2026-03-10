use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::domain::OrgMember;
use crate::dto::request::{CreateOrgMember, UpdateOrgMember};

use super::error::RepositoryError;
use super::filter::OrgMemberFilter;
use super::pagination::Page;

#[async_trait]
pub trait OrgMemberRepository: Send + Sync {
    async fn create(
        &self,
        org_id: Uuid,
        member: CreateOrgMember,
    ) -> Result<OrgMember, RepositoryError>;
    async fn get(
        &self,
        org_id: Uuid,
        member_id: Uuid,
    ) -> Result<Option<OrgMember>, RepositoryError>;
    async fn list(
        &self,
        filter: OrgMemberFilter,
        page: Page,
    ) -> Result<Vec<OrgMember>, RepositoryError>;
    async fn update(
        &self,
        org_id: Uuid,
        member_id: Uuid,
        update: UpdateOrgMember,
    ) -> Result<OrgMember, RepositoryError>;
    async fn delete(&self, org_id: Uuid, member_id: Uuid) -> Result<(), RepositoryError>;
}

pub struct OrgMemberRepositoryImpl<'a> {
    pool: &'a PgPool,
}

impl<'a> OrgMemberRepositoryImpl<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> OrgMemberRepository for OrgMemberRepositoryImpl<'a> {
    async fn create(
        &self,
        org_id: Uuid,
        member: CreateOrgMember,
    ) -> Result<OrgMember, RepositoryError> {
        let result = sqlx::query_as!(
            OrgMember,
            r#"
            INSERT INTO auth.orgs_people (org_id, member_id, role)
            VALUES ($1, $2, $3)
            RETURNING org_id, member_id, role, created_at, updated_at
            "#,
            org_id,
            member.member_id,
            member.role
        )
        .fetch_one(self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_foreign_key_violation() {
                    return RepositoryError::ConstraintViolation(
                        "Invalid org_id or member_id".to_string(),
                    );
                }
                if db_err.is_unique_violation() {
                    return RepositoryError::ConstraintViolation(
                        "Member already exists".to_string(),
                    );
                }
            }
            RepositoryError::Database(e)
        })?;

        Ok(result)
    }

    async fn get(
        &self,
        org_id: Uuid,
        member_id: Uuid,
    ) -> Result<Option<OrgMember>, RepositoryError> {
        let member = sqlx::query_as!(
            OrgMember,
            r#"
            SELECT org_id, member_id, role, created_at, updated_at
            FROM auth.orgs_people
            WHERE org_id = $1 AND member_id = $2
            "#,
            org_id,
            member_id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(member)
    }

    async fn list(
        &self,
        filter: OrgMemberFilter,
        page: Page,
    ) -> Result<Vec<OrgMember>, RepositoryError> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT org_id, member_id, role, created_at, updated_at FROM auth.orgs_people WHERE 1=1",
        );

        if let Some(org_id) = filter.org_id {
            query_builder.push(" AND org_id = ");
            query_builder.push_bind(org_id);
        }

        if let Some(member_id) = filter.member_id {
            query_builder.push(" AND member_id = ");
            query_builder.push_bind(member_id);
        }

        // Apply cursor-based pagination with custom ID column (Relay spec compliant)
        pagination::apply_cursor_pagination(&mut query_builder, &page, Some("member_id"), None)?;

        let mut members = query_builder
            .build_query_as::<OrgMember>()
            .fetch_all(self.pool)
            .await?;

        // Reverse results if backward pagination to maintain consistent ordering
        pagination::reverse_if_backward(&mut members, &page);

        Ok(members)
    }

    async fn update(
        &self,
        org_id: Uuid,
        member_id: Uuid,
        update: UpdateOrgMember,
    ) -> Result<OrgMember, RepositoryError> {
        let member = sqlx::query_as!(
            OrgMember,
            r#"
            UPDATE auth.orgs_people
            SET role = $1
            WHERE org_id = $2 AND member_id = $3
            RETURNING org_id, member_id, role, created_at, updated_at
            "#,
            update.role,
            org_id,
            member_id
        )
        .fetch_optional(self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(member)
    }

    async fn delete(&self, org_id: Uuid, member_id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM auth.orgs_people
            WHERE org_id = $1 AND member_id = $2
            "#,
            org_id,
            member_id
        )
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }
}
