#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use crate::trig::{TrignometryParams, Trigonometry};
use core::{borrow::BorrowMut, market::PhantomData};
use stylus_sdk::sol_storage::StorageSignedU256;
use stylus_sdk::sol_storage::StorageU256;
use stylus_sdk::sol_storage::StorageVec;
use stylus_sdk::{alloy_primitives::U256, prelude::*};

sol_storage! {
    
}