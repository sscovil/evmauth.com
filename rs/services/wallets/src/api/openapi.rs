use utoipa::OpenApi;

use crate::dto::request::{CreateEntityAppWallet, CreateEntityWallet};
use crate::dto::response::{EntityAppWalletResponse, EntityWalletResponse};

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
        // Entity wallets
        crate::api::handlers::entity_wallets::get_org_wallet,
        // Entity app wallets
        crate::api::handlers::entity_app_wallets::list_my_wallets,
        crate::api::handlers::entity_app_wallets::get_my_wallet,
    ),
    components(
        schemas(
            // Request DTOs
            CreateEntityWallet,
            CreateEntityAppWallet,
            // Response DTOs
            EntityWalletResponse,
            EntityAppWalletResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "entity_wallets", description = "Entity wallet endpoints"),
        (name = "entity_app_wallets", description = "Entity app wallet endpoints")
    )
)]
pub struct ApiDoc;
