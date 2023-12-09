 #![no_main]
 #![no_std]
 extern crate alloc;
 
 #[global_allocator]
 static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
 
 use alloc::{string::String, vec::Vec};
 use core::marker::PhantomData;
 use stylus_sdk::{
     alloy_primitives::{Address, U256, U64, bytes},
     alloy_sol_types::{sol, SolError},
     evm, msg,
     prelude::*,
     stylus_proc::entrypoint,
 };
 use crate StakeManager::StakeManager;
 mod StakeManager;
 use crate SenderCreator::SenderCreator;
 mod SenderCreator;
 use crate Helper::ValidationData;
 mod Helper;
 use crate NonceManager::NonceManager;
 mod NonceManager;
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

    pub trait EntryPointData {
        // internal value used during simulation: need to query aggregator.
        const SIMULATE_FIND_AGGREGATOR: &'static Address;
        // marker for inner call revert on out of gas
        const INNER_OUT_OF_GAS: &'static bytes;
        const REVERT_REASON_MAX_LEN: &'static U256;
        /**
     * for simulation purposes, validateUserOp (and validatePaymasterUserOp) must return this value
     * in case of signature failure, instead of revert.
     */
        const SIG_VALIDATION_FAILED: &'static U256;
    }
    /**
 * the interface exposed by a paymaster contract, who agrees to pay the gas for user's operations.
 * a paymaster must hold a stake to cover the required entrypoint stake and also the gas for the transaction.
 */
interface IPaymaster {

    enum PostOpMode {
        opSucceeded, // user op succeeded
        opReverted, // user op reverted. still has to pay for gas.
        postOpReverted //user op succeeded, but caused postOp to revert. Now it's a 2nd call, after user's op was deliberately reverted.
    }

    /**
     * payment validation: check if paymaster agrees to pay.
     * Must verify sender is the entryPoint.
     * Revert to reject this request.
     * Note that bundlers will reject this method if it changes the state, unless the paymaster is trusted (whitelisted)
     * The paymaster pre-pays using its deposit, and receive back a refund after the postOp method returns.
     * @param userOp the user operation
     * @param userOpHash hash of the user's request data.
     * @param maxCost the maximum cost of this transaction (based on maximum gas and gas price from userOp)
     * @return context value to send to a postOp
     *      zero length to signify postOp is not required.
     * @return validationData signature and time-range of this operation, encoded the same as the return value of validateUserOperation
     *      <20-byte> sigAuthorizer - 0 for valid signature, 1 to mark signature failure,
     *         otherwise, an address of an "authorizer" contract.
     *      <6-byte> validUntil - last timestamp this operation is valid. 0 for "indefinite"
     *      <6-byte> validAfter - first timestamp this operation is valid
     *      Note that the validation code cannot use block.timestamp (or block.number) directly.
     */
    function validatePaymasterUserOp(UserOperation calldata userOp, bytes32 userOpHash, uint256 maxCost)
    external returns (bytes memory context, uint256 validationData);

    /**
     * post-operation handler.
     * Must verify sender is the entryPoint
     * @param mode enum with the following options:
     *      opSucceeded - user operation succeeded.
     *      opReverted  - user op reverted. still has to pay for gas.
     *      postOpReverted - user op succeeded, but caused postOp (in mode=opSucceeded) to revert.
     *                       Now this is the 2nd call, after user's op was deliberately reverted.
     * @param context - the context value returned by validatePaymasterUserOp
     * @param actualGasCost - actual gas used so far (without this postOp call).
     */
    function postOp(PostOpMode mode, bytes calldata context, uint256 actualGasCost) external;
}

// Interface IEntryPoint is not implemented yet due to multiple internal interface imports in it
     
 }
 
 sol_storage! {
        #[entrypoint]
     pub struct EntryPoint {

     }

     pub struct MemoryUserOp {
        sender: Address,
        nonce: U256,
        call_gas_limit: U256,
        verification_gas_limit: U256,
        pre_verification_gas: U256,
        paymaster: Address,
        max_fee_per_gas: U256,
        max_priority_fee_per_gas: U256,
     }

     pub struct UserOpInfo {
        user_op_hash: bytes,
        prefund: U256,
        context_of_set: U256,
        pre_op_gas: U256
     }
 }
 
 impl EntryPoint {
/**
     * compensate the caller's beneficiary address with the collected fees of all UserOperations.
     * @param beneficiary the address to receive the fees
     * @param amount amount to transfer.
     */
     pub fn compensate(&mut self, beneficiary: Address, amount: U256) -> Result<(), Vec<u8>> {
         let entryPoint = self.entryPoint;
         let sender = msg.sender;
         if sender != entryPoint {
             return Err("NOT_FROM_ENTRY_POINT".into());
         }
         let balance = self.balance;
         if balance < amount {
             return Err("INSUFFICIENT_BALANCE".into());
         }
         self.balance -= amount;
         beneficiary.transfer(amount);
         Ok(())
     }
     /**
     * execute a user op
     * @param opIndex index into the opInfo array
     * @param userOp the userOp to execute
     * @param opInfo the opInfo filled by validatePrepayment for this userOp.
     * @return collected the total amount this userOp paid.
     */
     pub fn execute_user_op(&mut self, op_index: U256, user_op: UserOperation, op_info: OpInfo) -> Result<U256, Vec<u8>> {
         let entryPoint = self.entryPoint;
         let sender = msg.sender;
         if sender != entryPoint {
             return Err("NOT_FROM_ENTRY_POINT".into());
         }
         let op_info = self.opInfo[op_index];
         let collected = self.execute_user_op(op_index, user_op, op_info);
         Ok(collected)
     }
     /**
     * inner function to handle a UserOperation.
     * Must be declared "external" to open a call context, but it can only be called by handleOps.
     */
     pub fn inner_handle_op(&mut self, user_op: UserOperation, beneficiary: Address) -> Result<(), Vec<u8>> {
         let entryPoint = self.entryPoint;
         let sender = msg.sender;
         if sender != entryPoint {
             return Err("NOT_FROM_ENTRY_POINT".into());
         }
         let beneficiary = self.beneficiary;
         let collected = self.inner_handle_op(user_op, beneficiary);
         Ok(())
     }

     /**
     * copy general fields from userOp into the memory opInfo structure.
     */

     pub fn copy_user_op_to_memory(&mut self, user_op: UserOperation) -> Result<MemoryUserOp, Vec<u8>> {
         let entryPoint = self.entryPoint;
         let sender = msg.sender;
         if sender != entryPoint {
             return Err("NOT_FROM_ENTRY_POINT".into());
         }
         let memory_user_op = self.copy_user_op_to_memory(user_op);
         Ok(memory_user_op)
     }
      /**
     * Simulate a call to account.validateUserOp and paymaster.validatePaymasterUserOp.
     * @dev this method always revert. Successful result is ValidationResult error. other errors are failures.
     * @dev The node must also verify it doesn't use banned opcodes, and that it doesn't reference storage outside the account's data.
     * @param userOp the user operation to validate.
     */

     pub fn simulate_validation(&mut self, user_op: UserOperation) -> Result<(), Vec<u8>> {
         let entryPoint = self.entryPoint;
         let sender = msg.sender;
         if sender != entryPoint {
             return Err("NOT_FROM_ENTRY_POINT".into());
         }
         self.simulate_validation(user_op);
         Ok(())
     }

     pub fn get_required_prefund(&mut self, user_op: UserOperation) -> Result<U256, Vec<u8>> {
         let entryPoint = self.entryPoint;
         let sender = msg.sender;
         if sender != entryPoint {
             return Err("NOT_FROM_ENTRY_POINT".into());
         }
         let required_prefund = self.get_required_prefund(user_op);
         Ok(required_prefund)
     }
     // create the sender's contract if needed.
     pub fn create_sender_if_needed( &mut self, sender: Address) -> Result<(), Vec<u8>> {
         let entryPoint = self.entryPoint;
         let sender = msg.sender;
         let init_code = Vec::new();
         let sender = self.create_sender_if_needed(sender);
         Ok(())
     }
     /**
     * Get counterfactual sender address.
     *  Calculate the sender contract address that will be generated by the initCode and salt in the UserOperation.
     * this method always revert, and returns the address in SenderAddressResult error
     * @param initCode the constructor code to be passed into the UserOperation.
     */

     pub fn get_user_address(&mut self, init_code: Vec<u8>) -> Result<Address, Vec<u8>> {
         let entryPoint = self.entryPoint;
         let sender = msg.sender;
         let user_address = self.get_user_address(init_code);
         Ok(user_address)
     }

     pub fn simulation_only_validators(&mut self, user_op: UserOperation, user_op_hash: U256, missing_account_funds: U256) -> Result<(), Vec<u8>> {
         let entryPoint = self.entryPoint;
         let sender = msg.sender;
         let validation_data = self.simulation_only_validators(user_op, user_op_hash, missing_account_funds);
         Ok(())
     }
     
     /**
    * Called only during simulation.
    * This function always reverts to prevent warm/cold storage differentiation in simulation vs execution.
    */

    pub fn validate_sender_and_paymaster( &mut self, user_op: UserOperation, user_op_hash: U256, missing_account_funds: U256) -> Result<(), Vec<u8>> {
        let entryPoint = self.entryPoint;
        let sender = msg.sender;
        let validation_data = self.validate_sender_and_paymaster(user_op, user_op_hash, missing_account_funds);
        Ok(())
    }

    /**
     * call account.validateUserOp.
     * revert (with FailedOp) in case validateUserOp reverts, or account didn't send required prefund.
     * decrement account's deposit if needed
     */

     pub fn validate_account_repayment( &mut self, user_op: UserOperation, user_op_hash: U256, missing_account_funds: U256) -> Result<(), Vec<u8>> {
         let entryPoint = self.entryPoint;
         let sender = msg.sender;
         let validation_data = self.validate_account_repayment(user_op, user_op_hash, missing_account_funds);
         Ok(())
     }
     /**
     * In case the request has a paymaster:
     * Validate paymaster has enough deposit.
     * Call paymaster.validatePaymasterUserOp.
     * Revert with proper FailedOp in case paymaster reverts.
     * Decrement paymaster's deposit
     **/

     pub fn validate_paymaster_prepayment( &mut self, user_op: UserOperation, user_op_hash: U256, missing_account_funds: U256) -> Result<(), Vec<u8>> {
         let entryPoint = self.entryPoint;
         let sender = msg.sender;
         let validation_data = self.validate_paymaster_repayment(user_op, user_op_hash, missing_account_funds);
         Ok(())
     }
     /**
     * revert if either account validationData or paymaster validationData is expired
     */

     pub fn validate_account_and_paymaster_validation_data( &mut self, user_op: UserOperation, user_op_hash: U256, missing_account_funds: U256) -> Result<(), Vec<u8>> {
         let entryPoint = self.entryPoint;
         let sender = msg.sender;
         let validation_data = self.validate_account_and_paymaster_validation_data(user_op, user_op_hash, missing_account_funds);
         Ok(())
     }

     pub fn get_validation_data( &mut self, user_op: UserOperation, user_op_hash: U256, missing_account_funds: U256) -> Result<ValidationData, Vec<u8>> {
         let entryPoint = self.entryPoint;
         let sender = msg.sender;
         let validation_data = self.get_validation_data(user_op, user_op_hash, missing_account_funds);
         Ok(validation_data)
     }

 }
 #[external]
 impl EntryPoint {
     /**
     * Execute a batch of UserOperations.
     * no signature aggregator is used.
     * if any account requires an aggregator (that is, it returned an aggregator when
     * performing simulateValidation), then handleAggregatedOps() must be used instead.
     * @param ops the operations to execute
     * @param beneficiary the address to receive the fees
     */
     pub fn handle_ops(&mut self, ops: Vec<UserOperation>, beneficiary: Address) -> Result<(), Vec<u8>> {
         let entryPoint = self.entryPoint;
         let sender = msg.sender;
         if sender != entryPoint {
             return Err("NOT_FROM_ENTRY_POINT".into());
         }
         let beneficiary = self.beneficiary;
         let collected = self.handle_ops(ops, beneficiary);
         Ok(())
     }

      /**
     * Execute a batch of UserOperation with Aggregators
     * @param opsPerAggregator the operations to execute, grouped by aggregator (or address(0) for no-aggregator accounts)
     * @param beneficiary the address to receive the fees
     */

     pub fn handle_aggregated_ops(&mut self, ops_per_aggregator: Vec<(Address, Vec<UserOperation>)>, beneficiary: Address) -> Result<(), Vec<u8>> {
         let entryPoint = self.entryPoint;
         let sender = msg.sender;
         if sender != entryPoint {
             return Err("NOT_FROM_ENTRY_POINT".into());
         }
         let beneficiary = self.beneficiary;
         let collected = self.handle_aggregated_ops(ops_per_aggregator, beneficiary);
         Ok(())
     }
  
     /// @inheritdoc IEntryPoint

     pub fn simulated_ops(&mut self, ops: Vec<UserOperation>, beneficiary: Address) -> Result<(), Vec<u8>> {
         let entryPoint = self.entryPoint;
         let sender = msg.sender;
         if sender != entryPoint {
             return Err("NOT_FROM_ENTRY_POINT".into());
         }
         let beneficiary = self.beneficiary;
         let collected = self.simulated_ops(ops, beneficiary);
         Ok(())
     }

 /**
     * generate a request Id - unique identifier for this request.
     * the request ID is a hash over the content of the userOp (except the signature), the entrypoint and the chainid.
     */

     pub fn get_user_op_hash(&mut self, user_op: UserOperation) -> Result<U256, Vec<u8>> {
         let entryPoint = self.entryPoint;
         let sender = msg.sender;
         if sender != entryPoint {
             return Err("NOT_FROM_ENTRY_POINT".into());
         }
         let user_op_hash = self.get_user_op_hash(user_op);
         Ok(user_op_hash)
     }

 
 }