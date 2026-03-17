use utoipa::OpenApi;

use crate::api::handlers::accounts::{AccountsQuery, AccountsResponse, TokenBalance};
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
        description = "App registration, contract deployment, role grant management, and on-chain account queries"
    ),
    paths(
        // Health
        crate::api::handlers::health::health_check,
        // Accounts
        crate::api::handlers::accounts::get_account,
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
            AccountsQuery,
            AccountsResponse,
            TokenBalance,
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
        (name = "accounts", description = "On-chain account balance queries"),
        (name = "app_registrations", description = "App registration management"),
        (name = "contracts", description = "Contract deployment and role management")
    )
)]
pub struct ApiDoc;
