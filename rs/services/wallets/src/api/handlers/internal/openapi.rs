use utoipa::OpenApi;

use crate::dto::request::person_turnkey_ref::PasskeyAttestationParam;
use crate::dto::request::{CreateOrgWallet, CreatePersonAppWallet, CreatePersonTurnkeyRef};
use crate::dto::response::{OrgWalletResponse, PersonAppWalletResponse, PersonTurnkeyRefResponse};

use super::signing::SignRequest;

/// OpenAPI documentation for Internal API endpoints
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::handlers::internal::person_sub_orgs::create_person_sub_org,
        crate::api::handlers::internal::org_wallets::create_org_wallet,
        crate::api::handlers::internal::org_wallets::get_org_wallet_internal,
        crate::api::handlers::internal::person_app_wallets::create_person_app_wallet,
        crate::api::handlers::internal::person_app_wallets::get_person_app_wallet,
        crate::api::handlers::internal::signing::sign_payload,
    ),
    components(
        schemas(
            CreateOrgWallet,
            CreatePersonTurnkeyRef,
            PasskeyAttestationParam,
            CreatePersonAppWallet,
            SignRequest,
            OrgWalletResponse,
            PersonTurnkeyRefResponse,
            PersonAppWalletResponse,
        )
    ),
    tags(
        (name = "internal/person_sub_orgs", description = "Internal person sub-org management endpoints"),
        (name = "internal/org_wallets", description = "Internal org wallet management endpoints"),
        (name = "internal/person_app_wallets", description = "Internal person app wallet management endpoints"),
        (name = "internal/signing", description = "Internal signing endpoints")
    )
)]
pub struct InternalApiDoc;
