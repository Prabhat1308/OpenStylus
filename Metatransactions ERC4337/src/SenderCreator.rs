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
 
 sol_storage! {
    #[entrypoint]
    pub struct SenderCreator {

    }
}


//Look over this again after learning assembly
 impl SenderCreator {
    fn create_sender(&mut self, init_code: Vec<u8>) -> Result<Address, Vec<u8>> {
        let factory: Address = init_code[0..20].try_into().unwrap();
        let init_call_data = &init_code[20..];
    
        let mut sender: Address = [0; 20];
        let mut success: bool;
    
        unsafe {
            asm!(
                "call {0}, {1}, 0, {2}, {3}, 0, 32",
                inout(reg) success => _,
                inout(reg) factory => _,
                in(reg) init_call_data.as_ptr(),
                in(size_of_val(init_call_data) as u64)
            );
            asm!("mstore({0}, address())", inout(reg) sender.as_mut_ptr());
        }
    
        if !success {
            sender = [0; 20];
        }
    
        sender
    }
 
 }