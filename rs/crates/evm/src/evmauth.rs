use std::time::Duration;

use alloy::primitives::{Address, Bytes, FixedBytes, U256, Uint};
use alloy::sol;
use alloy::sol_types::SolCall;

use crate::EvmError;
use crate::client::EvmClient;

const RPC_TIMEOUT: Duration = Duration::from_secs(10);

// Generate type-safe bindings for the subset of EVMAuth6909 we need
sol! {
    #[sol(rpc)]
    interface IEVMAuth6909 {
        // ERC-6909 queries
        function balanceOf(address account, uint256 id) external view returns (uint256);
        function isOperator(address owner, address spender) external view returns (bool);

        // ERC-6909 token operations
        function mint(address to, uint256 id, uint256 amount) external;
        function setOperator(address operator, bool approved) external returns (bool);

        // Initialization (called via BeaconProxy constructor init_data)
        struct RoleGrant {
            bytes32 role;
            address account;
        }
        function initialize(uint48 initialDelay, address initialDefaultAdmin, address payable initialTreasury, RoleGrant[] roleGrants, string uri_) external;

        // Role management
        function grantRole(bytes32 role, address account) external;
        function revokeRole(bytes32 role, address account) external;

        // Role constant accessors
        function TOKEN_MANAGER_ROLE() external view returns (bytes32);
        function MINTER_ROLE() external view returns (bytes32);
        function BURNER_ROLE() external view returns (bytes32);
        function ACCESS_MANAGER_ROLE() external view returns (bytes32);
        function TREASURER_ROLE() external view returns (bytes32);
    }
}

/// Well-known EVMAuth role identifiers, computed as `keccak256(role_name)`.
/// These match the values returned by the on-chain role constant accessors.
pub mod roles {
    use alloy::primitives::{FixedBytes, keccak256};

    pub fn token_manager_role() -> FixedBytes<32> {
        keccak256("TOKEN_MANAGER_ROLE")
    }
    pub fn minter_role() -> FixedBytes<32> {
        keccak256("MINTER_ROLE")
    }
    pub fn burner_role() -> FixedBytes<32> {
        keccak256("BURNER_ROLE")
    }
    pub fn access_manager_role() -> FixedBytes<32> {
        keccak256("ACCESS_MANAGER_ROLE")
    }
    pub fn treasurer_role() -> FixedBytes<32> {
        keccak256("TREASURER_ROLE")
    }

    /// All non-admin roles that the platform operator receives on newly deployed proxies.
    pub fn all_operator_roles() -> Vec<FixedBytes<32>> {
        vec![
            token_manager_role(),
            access_manager_role(),
            treasurer_role(),
            minter_role(),
            burner_role(),
        ]
    }

    /// Map a human-readable role name (e.g. "MINTER_ROLE") to its keccak256 bytes32 value.
    /// Returns `None` for unrecognized role names.
    pub fn role_name_to_bytes(name: &str) -> Option<FixedBytes<32>> {
        match name {
            "TOKEN_MANAGER_ROLE" => Some(token_manager_role()),
            "MINTER_ROLE" => Some(minter_role()),
            "BURNER_ROLE" => Some(burner_role()),
            "ACCESS_MANAGER_ROLE" => Some(access_manager_role()),
            "TREASURER_ROLE" => Some(treasurer_role()),
            _ => None,
        }
    }

    /// All valid role name strings for validation.
    pub const VALID_ROLE_NAMES: &[&str] = &[
        "TOKEN_MANAGER_ROLE",
        "MINTER_ROLE",
        "BURNER_ROLE",
        "ACCESS_MANAGER_ROLE",
        "TREASURER_ROLE",
    ];
}

impl EvmClient {
    /// Query the balance of `account` for token `token_id` on the platform contract.
    pub async fn balance_of(&self, account: Address, token_id: U256) -> Result<U256, EvmError> {
        let contract = IEVMAuth6909::new(self.platform_contract_address(), self.provider());
        let balance =
            tokio::time::timeout(RPC_TIMEOUT, contract.balanceOf(account, token_id).call())
                .await
                .map_err(|_| EvmError::Timeout("balance_of call timed out".to_string()))?
                .map_err(|e| EvmError::Contract(format!("balance_of call failed: {e}")))?;

        Ok(balance)
    }

    /// Check whether `spender` is an operator for `owner` on the platform contract.
    pub async fn is_operator(&self, owner: Address, spender: Address) -> Result<bool, EvmError> {
        let contract = IEVMAuth6909::new(self.platform_contract_address(), self.provider());
        let is_op = tokio::time::timeout(RPC_TIMEOUT, contract.isOperator(owner, spender).call())
            .await
            .map_err(|_| EvmError::Timeout("is_operator call timed out".to_string()))?
            .map_err(|e| EvmError::Contract(format!("is_operator call failed: {e}")))?;

        Ok(is_op)
    }

    /// ABI-encode a `mint(to, id, amount)` call for signing by the wallets service.
    pub fn encode_mint(to: Address, token_id: U256, amount: U256) -> Bytes {
        let call = IEVMAuth6909::mintCall {
            to,
            id: token_id,
            amount,
        };
        Bytes::from(call.abi_encode())
    }

    /// ABI-encode an `initialize(...)` call for use as BeaconProxy init_data.
    ///
    /// Grants all non-admin roles (TOKEN_MANAGER, ACCESS_MANAGER, TREASURER, MINTER, BURNER)
    /// to the `platform_operator` address. The `initial_default_admin` receives
    /// `DEFAULT_ADMIN_ROLE` via the OpenZeppelin initializer.
    pub fn encode_initialize(
        initial_default_admin: Address,
        initial_treasury: Address,
        platform_operator: Address,
        uri: &str,
    ) -> Bytes {
        let role_grants: Vec<IEVMAuth6909::RoleGrant> = roles::all_operator_roles()
            .into_iter()
            .map(|role| IEVMAuth6909::RoleGrant {
                role,
                account: platform_operator,
            })
            .collect();

        let call = IEVMAuth6909::initializeCall {
            initialDelay: Uint::from(0u64),
            initialDefaultAdmin: initial_default_admin,
            initialTreasury: initial_treasury,
            roleGrants: role_grants,
            uri_: uri.to_string(),
        };
        Bytes::from(call.abi_encode())
    }

    /// ABI-encode a `grantRole(role, account)` call for signing by the wallets service.
    pub fn encode_grant_role(role: FixedBytes<32>, account: Address) -> Bytes {
        let call = IEVMAuth6909::grantRoleCall { role, account };
        Bytes::from(call.abi_encode())
    }

    /// ABI-encode a `revokeRole(role, account)` call for signing by the wallets service.
    pub fn encode_revoke_role(role: FixedBytes<32>, account: Address) -> Bytes {
        let call = IEVMAuth6909::revokeRoleCall { role, account };
        Bytes::from(call.abi_encode())
    }

    /// Query the balance of `account` for a specific token on a given contract address
    /// (not necessarily the platform contract).
    pub async fn balance_of_contract(
        &self,
        contract: Address,
        account: Address,
        token_id: U256,
    ) -> Result<U256, EvmError> {
        let instance = IEVMAuth6909::new(contract, self.provider());
        let balance =
            tokio::time::timeout(RPC_TIMEOUT, instance.balanceOf(account, token_id).call())
                .await
                .map_err(|_| EvmError::Timeout("balance_of call timed out".to_string()))?
                .map_err(|e| EvmError::Contract(format!("balance_of call failed: {e}")))?;
        Ok(balance)
    }

    /// Check whether `spender` is an operator for `owner` on a given contract address
    /// (not necessarily the platform contract).
    pub async fn is_operator_on_contract(
        &self,
        contract: Address,
        owner: Address,
        spender: Address,
    ) -> Result<bool, EvmError> {
        let instance = IEVMAuth6909::new(contract, self.provider());
        let is_op = tokio::time::timeout(RPC_TIMEOUT, instance.isOperator(owner, spender).call())
            .await
            .map_err(|_| EvmError::Timeout("is_operator call timed out".to_string()))?
            .map_err(|e| EvmError::Contract(format!("is_operator call failed: {e}")))?;
        Ok(is_op)
    }

    /// Batch-query balances for multiple token IDs on a given contract.
    /// Returns `(token_id, balance)` pairs in the same order as input.
    pub async fn balances_for(
        &self,
        contract: Address,
        account: Address,
        token_ids: &[U256],
    ) -> Result<Vec<(U256, U256)>, EvmError> {
        let futs: Vec<_> = token_ids
            .iter()
            .map(|&token_id| async move {
                let balance = self
                    .balance_of_contract(contract, account, token_id)
                    .await?;
                Ok::<_, EvmError>((token_id, balance))
            })
            .collect();

        let results = futures::future::try_join_all(futs).await?;
        Ok(results)
    }
}
