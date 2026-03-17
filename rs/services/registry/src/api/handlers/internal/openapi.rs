use utoipa::OpenApi;

use crate::dto::response::{AppRegistrationResponse, ContractResponse};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::handlers::internal::app_registrations::get_app_by_client_id,
        crate::api::handlers::internal::contracts::get_contract_internal,
        crate::api::handlers::internal::contracts::get_contract_by_app_registration_id,
    ),
    components(
        schemas(
            AppRegistrationResponse,
            ContractResponse,
        )
    ),
    tags(
        (name = "internal/app_registrations", description = "Internal app registration lookup endpoints"),
        (name = "internal/contracts", description = "Internal contract lookup endpoints")
    )
)]
pub struct InternalApiDoc;
