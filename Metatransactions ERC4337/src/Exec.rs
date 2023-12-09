 #![no_main]
 #![no_std]
 extern crate alloc;
 /**
 * Utility functions helpful when making different kinds of contract calls in Solidity.
 */
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
 
 sol_storage! {
     pub struct Exec {
 }
 
 impl Exec {
    pub fn call(&mut self, to: Address, value: U256, data: Vec<u8>, tx_gas: U256) -> Result<bool, Vec<u8>> {
        let mut success: bool = false;
        let data_ptr = data.as_ptr();
    
        unsafe {
            asm!(
                "call {0}, {1}, {2}, {3}, {4}, 0, 0",
                inout(reg) tx_gas => _,
                inout(reg) to => _,
                in(reg) value,
                in(data_ptr),
                in(data.len() as u64),
                lateout(reg) success,
            );
        }
    
        success
    }
    
    pub fn staticcall(to: Address, data: Vec<u8>, tx_gas: U256) -> Result<bool, Vec<u8>> {
        let mut success: bool = false;
        let data_ptr = data.as_ptr();
    
        unsafe {
            asm!(
                "staticcall {0}, {1}, {2}, {3}, 0, 0",
                inout(reg) tx_gas => _,
                inout(reg) to => _,
                in(data_ptr),
                in(data.len() as u64),
                lateout(reg) success,
            );
        }
    
        success
    }
    
    fn delegate_call(to: Address, data: Vec<u8>, tx_gas: U256) -> Result<bool, Vec<u8>> {
        let mut success: bool = false;
        let data_ptr = data.as_ptr();
    
        unsafe {
            asm!(
                "delegatecall {0}, {1}, {2}, {3}, 0, 0",
                inout(reg) tx_gas => _,
                inout(reg) to => _,
                in(data_ptr),
                in(data.len() as u64),
                lateout(reg) success,
            );
        }
    
        success
    }
    
    pub fn get_return_data(max_len: usize) -> Result<Vec<u8>, Vec<u8>> {
        let mut return_data: Vec<u8> = vec![0; max_len];
    
        unsafe {
            asm!(
                "let len := returndatasize()
                 if gt(len, {0}) {{
                     len := {0}
                 }}
                 let ptr := add({1}, 0x20)
                 mstore(ptr, len)
                 returndatacopy(add(ptr, 0x20), 0, len)",
                in(max_len as u64),
                inout(reg) return_data,
            );
        }
    
        return_data
    }
    
    pub fn revert_with_data(return_data: Vec<u8>) {
        unsafe {
            asm!(
                "revert(add({0}, 32), mload(add({0}, 0x20)))",
                in(return_data),
            );
        }
    }
    
    pub fn call_and_revert(to: Address, data: Vec<u8>, max_len: usize) {
        let success = call(to, 0, data.clone(), gasleft());
        if !success {
            let return_data = get_return_data(max_len);
            revert_with_data(return_data);
        }
    }
 
 }