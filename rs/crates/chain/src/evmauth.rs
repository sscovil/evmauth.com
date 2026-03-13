use alloy::primitives::{Address, Bytes, U256};
use alloy::sol;
use alloy::sol_types::SolCall;

use crate::ChainError;
use crate::client::ChainClient;

// Generate type-safe bindings for the subset of EVMAuth6909 we need
sol! {
    #[sol(rpc)]
    interface IEVMAuth6909 {
        function balanceOf(address account, uint256 id) external view returns (uint256);
        function isOperator(address owner, address spender) external view returns (bool);
        function mint(address to, uint256 id, uint256 amount) external;
        function setOperator(address operator, bool approved) external returns (bool);
    }
}

impl ChainClient {
    /// Query the balance of `account` for token `token_id` on the platform contract.
    pub async fn balance_of(&self, account: Address, token_id: U256) -> Result<U256, ChainError> {
        let contract = IEVMAuth6909::new(self.platform_contract_address(), self.provider());
        let balance = contract
            .balanceOf(account, token_id)
            .call()
            .await
            .map_err(|e| ChainError::Contract(format!("balanceOf call failed: {e}")))?;

        Ok(balance)
    }

    /// Check whether `spender` is an operator for `owner` on the platform contract.
    pub async fn is_operator(&self, owner: Address, spender: Address) -> Result<bool, ChainError> {
        let contract = IEVMAuth6909::new(self.platform_contract_address(), self.provider());
        let is_op = contract
            .isOperator(owner, spender)
            .call()
            .await
            .map_err(|e| ChainError::Contract(format!("isOperator call failed: {e}")))?;

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

    /// ABI-encode a `setOperator(operator, approved)` call for signing by the wallets service.
    pub fn encode_set_operator(operator: Address, approved: bool) -> Bytes {
        let call = IEVMAuth6909::setOperatorCall { operator, approved };
        Bytes::from(call.abi_encode())
    }
}
