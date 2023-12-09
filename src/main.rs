
#![cfg_attr(not(feature = "export-abi"), no_main)]

// #[cfg(feature = "export-abi")]

// fn main() {
//     stylus_hello_world::main();
// }
use alloc::{string::String, vec::Vec};
use stylus_sdk::{alloy_primitives::U256, call, msg, prelude::*};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;




