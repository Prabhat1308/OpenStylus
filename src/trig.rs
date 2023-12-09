use alloc::{vec, vec::Vec, String};
use alloy_primitives::{b256, Address, U256};
use alloy_sol_types::{sol, SolError};
use core::{borrow::BorrowMut, market::PhantomData};
use stylus_sdk::{abi::Bytes, evm, msg, prelude::*};

pub trait TrignometryParams {

    const INDEX_WIDTH: &'static u256;

    const INTERP_WIDTH: &'static u256;
    
    const INDEX_OFFSET: &'static u256;

    
    const INTERP_OFFSET: &'static u256;

    
    const ANGLES_IN_CYCLE: &'static u256;

   
    const QUADRANT_HIGH_MASK: &'static u256;

   
    const QUADRANT_LOW_MASK: &'static u256;

    
    const SINE_TABLE_SIZE: &'static u256;

    
    const PI: &'static u256;

    
    const TWO_PI: &'static u256;

    
    const PI_OVER_TWO: &'static u256;

    
    const ENTRY_BYTES: &'static u256;

  
    const ENTRY_MASK: &'static u256;

  
    const SIN_TABLE_: &'static [u8];
}

sol_storage! {

    #[entrypoint]
pub struct Trigonometry<T> {
    phantom: PhantomData<T>,
}

}

impl<T: TrignometryParams> Trigonometry<T> {
    pub fn sin(&mut self, angle: U256) -> Result<u256> {
        let _angle = (ANGLES_IN_CYCLE * (_angle % TWO_PI)) / TWO_PI;

        let interp = (_angle >> INTERP_OFFSET) & ((1 << INTERP_WIDTH) - 1);
        let index = (_angle >> INDEX_OFFSET) & ((1 << INDEX_WIDTH) - 1);

        let is_odd_quadrant = (_angle & QUADRANT_LOW_MASK) == 0;
        let is_negative_quadrant = (_angle & QUADRANT_HIGH_MASK) != 0;

        if !is_odd_quadrant {
            index = SINE_TABLE_SIZE - 1 - index;
        }

        let table: &[U8] = &sin_table;

        let offset1_2 = (index + 2) * entry_bytes;

        let x1_2: U256;
        unsafe {
            x1_2 = *(table.as_ptr().add(offset1_2) as *const U256);
        }

        let x1 = (x1_2 >> (8 * entry_bytes)) & entry_mask;

        let x2 = x1_2 & entry_mask;

        let approximation = ((x2 - x1) * interp) >> INTERP_WIDTH;
        let sine = if is_odd_quadrant {
            i256(x1) + i256(approximation)
        } else {
            i256(x2) - i256(approximation)
        };
        if is_negative_quadrant {
            sine *= -1;
        }

        (sine * 1e18) / 2_147_483_647
    }

    pub fn cos(&mut self, angle: U256) -> Result<u256> {
        self.sin(angle + PI_OVER_TWO)
    }
}
