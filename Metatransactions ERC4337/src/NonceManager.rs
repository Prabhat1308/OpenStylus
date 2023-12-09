/**
 * nonce management functionality
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

sol_interface! {
    interface INonceManager {

        /**
         * Return the next nonce for this sender.
         * Within a given key, the nonce values are sequenced (starting with zero, and incremented by one on each userop)
         * But UserOp with different keys can come with arbitrary order.
         *
         * @param sender the account address
         * @param key the high 192 bit of the nonce
         * @return nonce a full nonce to pass for next UserOp with this sender.
         */
        function getNonce(address sender, uint192 key)
        external view returns (uint256 nonce);
    
        /**
         * Manually increment the nonce of the sender.
         * This method is exposed just for completeness..
         * Account does NOT need to call it, neither during validation, nor elsewhere,
         * as the EntryPoint will update the nonce regardless.
         * Possible use-case is call it with various keys to "initialize" their nonces to one, so that future
         * UserOperations will not pay extra for the first transaction with a given key.
         */
        function incrementNonce(uint192 key) external;
    }
    
}

sol_storage! {
    pub struct NonceManager {/**
     * The next valid sequence number for a given nonce key.
     */
     mapping(address => mapping(uint192 => uint256)) public nonceSequenceNumber;
    }
}

impl NonceManager {
    pub fn get_nonce(&mut self, sender: Address, key: U256) -> Result<U256, Vec<u8>> {
        let nonce = self.nonceSequenceNumber[sender][key];
        Ok(nonce)
    }

    // allow an account to manually increment its own nonce.
    // (mainly so that during construction nonce can be made non-zero,
    // to "absorb" the gas cost of first nonce increment to 1st transaction (construction),
    // not to 2nd transaction)

    pub fn increment_nonce(&mut self, key: U256) -> Result<(), Vec<u8>> {
        self.nonceSequenceNumber[msg::sender()][key] += 1;
        Ok(())
    }

     /**
     * validate nonce uniqueness for this account.
     * called just after validateUserOp()
     */
     pub fn validate_and_update_nonce(sender: Address, nonce: U256) -> Reslut<bool, Vec<u8>> {
        let key = (nonce >> 64) as U256;
        let seq = nonce as U64;

        let matching = self.nonceSequenceNumber[msg::sender()][key].get() + U256::from(2);
        
        if matching == seq {
            return Ok(true);
        }
    }

}