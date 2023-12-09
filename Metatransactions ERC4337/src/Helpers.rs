/**
 * Returned data from validateUserOp.
 * validateUserOp returns a uint256, with is created by `_packedValidationData` and
 * parsed by `_parseValidationData`.
 * @param aggregator  - address(0) - The account validated the signature by itself.
 *                      address(1) - The account failed to validate the signature.
 *                      otherwise - This is an address of a signature aggregator that must
 *                                  be used to validate the signature.
 * @param validAfter  - This UserOp is valid only after this timestamp.
 * @param validaUntil - This UserOp is valid only up to this timestamp.
 */
 #![no_main]
 #![no_std]
 extern crate alloc;
 
 #[global_allocator]
 static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
 
 use alloc::{string::String, vec::Vec};
 use core::marker::PhantomData;
 use stylus_sdk::{
     alloy_primitives::{Address, U256},
     alloy_sol_types::{sol, SolError},
     evm, msg,
     prelude::*,
     stylus_proc::entrypoint,
 };
 
 sol_storage! {
     #[entrypoint]
     pub struct ValidationData {
        address aggregator;
        uint48 validAfter;
        uint48 validUntil;
     }
     
 }


impl ValidationData {
    fn parse_validation_data(&mut self, validation_data: U256) -> Result<ValidationData, Vec<u8>> {
        let aggregator = Address::from_slice(&validation_data.to_le_bytes()[0..20]);
        let valid_until = (validation_data >> 160) as U256;
        let valid_after = ((validation_data >> (48 + 160)) & ((1 << 48) - 1)) as U256;
    
        let valid_until = if valid_until == 0 {
            U256::MAX
        } else {
            valid_until
        };
    
        Ok(ValidationData {
            aggregator,
            valid_after,
            valid_until,
        })
    }

    fn intersect_time_range(
        &mut self,
        validation_data: U256,
        paymaster_validation_data: U256,
    ) -> Result<ValidationData, Vec<u8>>{
        let account_validation_data = parse_validation_data(validation_data);
        let pm_validation_data = parse_validation_data(paymaster_validation_data);
    
        let mut aggregator = account_validation_data.aggregator;
        if aggregator == [0; 20] {
            aggregator = pm_validation_data.aggregator;
        }
    
        let mut valid_after = account_validation_data.valid_after;
        let mut valid_until = account_validation_data.valid_until;
        let pm_valid_after = pm_validation_data.valid_after;
        let pm_valid_until = pm_validation_data.valid_until;
    
        if valid_after < pm_valid_after {
            valid_after = pm_valid_after;
        }
        if valid_until > pm_valid_until {
            valid_until = pm_valid_until;
        }
    
        Ok(ValidationData {
            aggregator,
            valid_after,
            valid_until,
        })
    }


fn pack_validation_data(&mut self, data: ValidationData) -> Result<U256, Vec<u8>> {
    let result = U256::from(data.aggregator)
        | (U256::from(data.valid_until) << 160)
        | (U256::from(data.valid_after) << (160 + 48));

    Ok(result)
}

fn pack_validation_data_without_aggregator(&mut self, sig_failed: bool, valid_until: U256, valid_after: U256) -> Result<U256, Vec<u8>> {
    let result = (if sig_failed { 1 } else { 0 })
        | (u256::from(valid_until) << 160)
        | (u256::from(valid_after) << (160 + 48));

    Ok(result)
}

//implement this function after learning assembly in stylus
// /**
//  * keccak function over calldata.
//  * @dev copy calldata into memory, do keccak and drop allocated memory. Strangely, this is more efficient than letting solidity do it.
//  */
//  function calldataKeccak(bytes calldata data) pure returns (bytes32 ret) {
//     assembly {
//         let mem := mload(0x40)
//         let len := data.length
//         calldatacopy(mem, data.offset, len)
//         ret := keccak256(mem, len)
//     }
// }
}