use utoipa::OpenApi;

use crate::dto::request::entity_wallet::PasskeyAttestationParam;
use crate::dto::request::{CreateEntityAppWallet, CreateEntityWallet};
use crate::dto::response::{EntityAppWalletResponse, EntityWalletResponse};

use super::authenticators::CreateAuthenticatorsRequest;
use super::send_tx::{SendTxRequest, SendTxResponse};
use super::signing::SignRequest;

/// OpenAPI documentation for Internal API endpoints
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::handlers::internal::entity_wallets::create_entity_wallet,
        crate::api::handlers::internal::entity_wallets::get_entity_wallet,
        crate::api::handlers::internal::entity_app_wallets::create_entity_app_wallet,
        crate::api::handlers::internal::entity_app_wallets::get_entity_app_wallet,
        crate::api::handlers::internal::signing::sign_payload,
        crate::api::handlers::internal::send_tx::send_tx,
        crate::api::handlers::internal::authenticators::create_authenticators,
    ),
    components(
        schemas(
            CreateEntityWallet,
            PasskeyAttestationParam,
            CreateEntityAppWallet,
            CreateAuthenticatorsRequest,
            SignRequest,
            SendTxRequest,
            SendTxResponse,
            EntityWalletResponse,
            EntityAppWalletResponse,
        )
    ),
    tags(
        (name = "internal/entity_wallets", description = "Internal entity wallet management endpoints"),
        (name = "internal/entity_app_wallets", description = "Internal entity app wallet management endpoints"),
        (name = "internal/signing", description = "Internal signing endpoints"),
        (name = "internal/send_tx", description = "Internal transaction broadcasting endpoints"),
        (name = "internal/authenticators", description = "Internal authenticator management endpoints")
    )
)]
pub struct InternalApiDoc;
