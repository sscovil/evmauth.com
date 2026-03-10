use utoipa::OpenApi;

use crate::dto::request::{CreateOrgWallet, CreatePersonAppWallet, CreatePersonTurnkeyRef};
use crate::dto::response::{OrgWalletResponse, PersonAppWalletResponse};

/// OpenAPI documentation for the Wallets Service (public endpoints only)
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Wallets Service API",
        version = "1.0.0",
        description = "Wallet management service for Turnkey sub-org lifecycle, wallet creation, and signing operations"
    ),
    paths(
        // Health
        crate::api::handlers::health::health_check,
        // Org wallets
        crate::api::handlers::org_wallets::get_org_wallet,
        // Person wallets
        crate::api::handlers::person_wallets::list_my_wallets,
        crate::api::handlers::person_wallets::get_my_wallet,
    ),
    components(
        schemas(
            // Request DTOs
            CreateOrgWallet,
            CreatePersonTurnkeyRef,
            CreatePersonAppWallet,
            // Response DTOs
            OrgWalletResponse,
            PersonAppWalletResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "org_wallets", description = "Organization wallet endpoints"),
        (name = "person_wallets", description = "Person wallet endpoints")
    )
)]
pub struct ApiDoc;
