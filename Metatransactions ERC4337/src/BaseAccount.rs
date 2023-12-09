/**
 * Basic account implementation.
 * this contract provides the basic logic for implementing the IAccount interface  - validateUserOp
 * specific account implementation should inherit it and provide the account-specific logic
 */
#![no_main]
 #![no_std]
 extern crate alloc;
 
 #[global_allocator]
 static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
 
 use alloc::{string::String, vec::Vec};
 use core::marker::PhantomData;
 use stylus_sdk::{
     alloy_primitives::{Address, U256, U64},
     alloy_sol_types::{sol, SolError},
     evm, msg,
     prelude::*,
     stylus_proc::entrypoint,
 };
 use crate Helper::ValidationData;
 mod Helper;
 
 sol_interface! {
    interface IAccount {

        /**
         * Validate user's signature and nonce
         * the entryPoint will make the call to the recipient only if this validation call returns successfully.
         * signature failure should be reported by returning SIG_VALIDATION_FAILED (1).
         * This allows making a "simulation call" without a valid signature
         * Other failures (e.g. nonce mismatch, or invalid signature format) should still revert to signal failure.
         *
         * @dev Must validate caller is the entryPoint.
         *      Must validate the signature and nonce
         * @param userOp the operation that is about to be executed.
         * @param userOpHash hash of the user's request data. can be used as the basis for signature.
         * @param missingAccountFunds missing funds on the account's deposit in the entrypoint.
         *      This is the minimum amount to transfer to the sender(entryPoint) to be able to make the call.
         *      The excess is left as a deposit in the entrypoint, for future calls.
         *      can be withdrawn anytime using "entryPoint.withdrawTo()"
         *      In case there is a paymaster in the request (or the current deposit is high enough), this value will be zero.
         * @return validationData packaged ValidationData structure. use `_packValidationData` and `_unpackValidationData` to encode and decode
         *      <20-byte> sigAuthorizer - 0 for valid signature, 1 to mark signature failure,
         *         otherwise, an address of an "authorizer" contract.
         *      <6-byte> validUntil - last timestamp this operation is valid. 0 for "indefinite"
         *      <6-byte> validAfter - first timestamp this operation is valid
         *      If an account doesn't use time-range, it is enough to return SIG_VALIDATION_FAILED value (1) for signature failure.
         *      Note that the validation code cannot use block.timestamp (or block.number) directly.
         */
        function validateUserOp(UserOperation calldata userOp, bytes32 userOpHash, uint256 missingAccountFunds)
        external returns (uint256 validationData);
    }

    //problem in using IEntryPoint interface due to multiple internal interface imports in it
     
 }

//  using UserOperationLib for UserOperation; not implemented this yet, necessary!!
 
pub trait Erc20Params {
    const SIG_VALIDATION_FAILED: &'static U256;
}

 sol_storage! {
    #[entrypoint]
     pub struct BaseAccount {
     }
    }

impl BaseAccount {
    /**
     * ensure the request comes from the known entrypoint.
     */
    pub fn require_from_entry_point(&mut self) -> Result<(), Vec<u8>> {
        let entryPoint = self.entryPoint;
        let sender = msg.sender;
        if sender != entryPoint {
            return Err("NOT_FROM_ENTRY_POINT".into());
        }
        Ok(())
    }

     /**
     * validate the signature is valid for this message.
     * @param userOp validate the userOp.signature field
     * @param userOpHash convenient field: the hash of the request, to check the signature against
     *          (also hashes the entrypoint and chain id)
     * @return validationData signature and time-range of this operation
     *      <20-byte> sigAuthorizer - 0 for valid signature, 1 to mark signature failure,
     *         otherwise, an address of an "authorizer" contract.
     *      <6-byte> validUntil - last timestamp this operation is valid. 0 for "indefinite"
     *      <6-byte> validAfter - first timestamp this operation is valid
     *      If the account doesn't use time-range, it is enough to return SIG_VALIDATION_FAILED value (1) for signature failure.
     *      Note that the validation code cannot use block.timestamp (or block.number) directly.
     */

        pub fn validate_signature(&mut self, user_op: UserOperation, user_op_hash: U256) -> Result<U256, Vec<u8>> {
            let sig_authorizer = self.validate_signature(user_op, user_op_hash);
            Ok(sig_authorizer)
        }

        /**
     * Validate the nonce of the UserOperation.
     * This method may validate the nonce requirement of this account.
     * e.g.
     * To limit the nonce to use sequenced UserOps only (no "out of order" UserOps):
     *      `require(nonce < type(uint64).max)`
     * For a hypothetical account that *requires* the nonce to be out-of-order:
     *      `require(nonce & type(uint64).max == 0)`
     *
     * The actual nonce uniqueness is managed by the EntryPoint, and thus no other
     * action is needed by the account itself.
     *
     * @param nonce to validate
     *
     * solhint-disable-next-line no-empty-blocks
     */
    pub fn validate_nonce(&mut self, nonce: U64) -> Result<(), Vec<u8>> {
        Ok(())
    }
         /**
     * sends to the entrypoint (msg.sender) the missing funds for this transaction.
     * subclass MAY override this method for better funds management
     * (e.g. send to the entryPoint more than the minimum required, so that in future transactions
     * it will not be required to send again)
     * @param missingAccountFunds the minimum value this method should send the entrypoint.
     *  this value MAY be zero, in case there is enough deposit, or the userOp has a paymaster.
     */
    pub fn pay_prefund(&mut self, missing_account_funds: U256) -> Result<(), Vec<u8>> {
        let entryPoint = self.entryPoint;
        let sender = msg.sender;
        let value = missing_account_funds;
        let data = Vec::new();
        let gas_limit = 0;
        let gas_price = 0;
        let result = evm::call(sender, entryPoint, value, data, gas_limit, gas_price);
        if result.is_err() {
            return Err("PAY_PREFUND_FAILED".into());
        }
        Ok(())
    }
}

 #[external]
 impl BaseAccount {
     /**
     * Return the account nonce.
     * This method returns the next sequential nonce.
     * For a nonce of a specific key, use `entrypoint.getNonce(account, key)`
     */

    pub fn get_nonce(&mut self, sender: Address, key: U256) -> Result<U256, Vec<u8>> {
        let nonce = self.nonceSequenceNumber[sender][key];
        Ok(nonce)
    }

    /**
     * return the entryPoint used by this account.
     * subclass should return the current entryPoint used by this account.
     */

     pub fn get_entry_point(&mut self) -> Result<Address, Vec<u8>> {
        let entryPoint = self.entryPoint;
        Ok(entryPoint)
     }
     /**
     * Validate user's signature and nonce.
     * subclass doesn't need to override this method. Instead, it should override the specific internal validation methods.
     */

     pub fn validate_user_op(&mut self, user_op: UserOperation, user_op_hash: U256, missing_account_funds: U256) -> Result<U256, Vec<u8>> {
        let validation_data = self.validate_user_op(user_op, user_op_hash, missing_account_funds);
        Ok(validation_data)
     }




    }
