use utoipa::OpenApi;

use crate::dto::request::{CreateAppRegistration, CreateContract, UpdateAppRegistration};
use crate::dto::response::{
    AppRegistrationResponse, ContractResponse, OperatorGrantResponse, SendTxResponse,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Registry Service API",
        version = "1.0.0",
        description = "App registration, contract deployment, and operator grant management"
    ),
    paths(
        // Health
        crate::api::handlers::health::health_check,
        // App registrations
        crate::api::handlers::app_registrations::create_app_registration,
        crate::api::handlers::app_registrations::list_app_registrations,
        crate::api::handlers::app_registrations::get_app_registration,
        crate::api::handlers::app_registrations::update_app_registration,
        crate::api::handlers::app_registrations::delete_app_registration,
        // Contracts
        crate::api::handlers::contracts::deploy_contract,
        crate::api::handlers::contracts::list_contracts,
        crate::api::handlers::contracts::get_contract,
        crate::api::handlers::contracts::grant_operator,
        crate::api::handlers::contracts::revoke_operator,
    ),
    components(
        schemas(
            CreateAppRegistration,
            UpdateAppRegistration,
            CreateContract,
            AppRegistrationResponse,
            ContractResponse,
            OperatorGrantResponse,
            SendTxResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "app_registrations", description = "App registration management"),
        (name = "contracts", description = "Contract deployment and operator management")
    )
)]
pub struct ApiDoc;
