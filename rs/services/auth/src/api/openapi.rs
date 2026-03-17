use pagination::PaginatedResponse;
use utoipa::OpenApi;

use crate::api::handlers::auth::{LoginRequest, PasskeyAttestation, SignupRequest, TokenResponse};
use crate::api::handlers::challenges::ChallengeResponse;
use crate::api::handlers::end_users::{CreateEndUserRequest, CreateEndUserResponse};
use crate::api::handlers::me::{CreateAuthenticatorRequest, UpdateMeRequest};
use crate::domain::OrgVisibility;
use crate::dto::request::{CreateOrg, CreateOrgMember, UpdateOrg, UpdateOrgMember, UpdatePerson};
use crate::dto::response::{OrgMemberResponse, OrgResponse, PersonResponse};

/// OpenAPI documentation for the Auth Service (public endpoints only)
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Auth Service API",
        version = "1.0.0",
        description = "Authentication and organization management service API"
    ),
    paths(
        // Health
        crate::api::handlers::health::health_check,
        // Auth
        crate::api::handlers::auth::signup,
        crate::api::handlers::auth::login,
        crate::api::handlers::auth::logout,
        // Challenges
        crate::api::handlers::challenges::create_challenge,
        // End-user onboarding
        crate::api::handlers::end_users::create_end_user,
        // Me
        crate::api::handlers::me::get_me,
        crate::api::handlers::me::update_me,
        crate::api::handlers::me::create_authenticator,
        // People
        crate::api::handlers::people::list_people,
        crate::api::handlers::people::get_person,
        crate::api::handlers::people::update_person,
        crate::api::handlers::people::delete_person,
        // Orgs
        crate::api::handlers::orgs::list_orgs,
        crate::api::handlers::orgs::get_org,
        crate::api::handlers::orgs::create_org,
        crate::api::handlers::orgs::update_org,
        crate::api::handlers::orgs::delete_org,
        // Org Members
        crate::api::handlers::org_members::list_org_members,
        crate::api::handlers::org_members::create_org_member,
        crate::api::handlers::org_members::update_org_member,
        crate::api::handlers::org_members::delete_org_member,
    ),
    components(
        schemas(
            // Auth DTOs
            SignupRequest,
            LoginRequest,
            TokenResponse,
            PasskeyAttestation,
            UpdateMeRequest,
            CreateAuthenticatorRequest,
            // Challenge DTOs
            ChallengeResponse,
            // End-user DTOs
            CreateEndUserRequest,
            CreateEndUserResponse,
            // Request DTOs
            UpdatePerson,
            CreateOrg,
            UpdateOrg,
            CreateOrgMember,
            UpdateOrgMember,
            // Response DTOs
            PersonResponse,
            OrgResponse,
            OrgMemberResponse,
            // Enums
            OrgVisibility,
            // Paginated responses
            PaginatedResponse<PersonResponse>,
            PaginatedResponse<OrgResponse>,
            PaginatedResponse<OrgMemberResponse>,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "auth", description = "Authentication endpoints"),
        (name = "end_users", description = "End-user onboarding endpoints"),
        (name = "me", description = "Current user profile endpoints"),
        (name = "people", description = "Person management endpoints"),
        (name = "orgs", description = "Organization management endpoints"),
        (name = "org_members", description = "Organization member management endpoints")
    )
)]
pub struct ApiDoc;
