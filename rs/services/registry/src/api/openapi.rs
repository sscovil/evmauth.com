use utoipa::OpenApi;

use crate::dto::request::{
    CreateAppRegistration, CreateContract, CreateRoleGrant, UpdateAppRegistration,
};
use crate::dto::response::{
    AppRegistrationResponse, ContractResponse, RoleGrantResponse, SendTxResponse,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Registry Service API",
        version = "1.0.0",
        description = "App registration, contract deployment, and role grant management"
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
        // Role grants
        crate::api::handlers::contracts::create_role_grant,
        crate::api::handlers::contracts::delete_role_grant,
        crate::api::handlers::contracts::list_role_grants,
    ),
    components(
        schemas(
            CreateAppRegistration,
            UpdateAppRegistration,
            CreateContract,
            CreateRoleGrant,
            AppRegistrationResponse,
            ContractResponse,
            RoleGrantResponse,
            SendTxResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "app_registrations", description = "App registration management"),
        (name = "contracts", description = "Contract deployment and role management")
    )
)]
pub struct ApiDoc;
