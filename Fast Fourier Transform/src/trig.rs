use alloc::{vec::Vec};
use core::{ market::PhantomData};
use stylus_sdk::{alloy_primitives::U256, alloy_primitives::I256,prelude::*};
use alloy_primitives::U8;

pub trait TrignometryParams {

    const INDEX_WIDTH: &'static U256;

    const INTERP_WIDTH: &'static U256;
    
    const INDEX_OFFSET: &'static U256;

    
    const INTERP_OFFSET: &'static U256;

    
    const ANGLES_IN_CYCLE: &'static U256;

   
    const QUADRANT_HIGH_MASK: &'static U256;

   
    const QUADRANT_LOW_MASK: &'static U256;

    
    const SINE_TABLE_SIZE: &'static U256;

    
    const PI: &'static U256;

    
    const TWO_PI: &'static U256;

    
    const PI_OVER_TWO: &'static U256;

    
    const ENTRY_BYTES: &'static U256;

  
    const ENTRY_MASK: &'static U256;

  
    const SIN_TABLE_: &'static [U8];
}

sol_storage! {

    #[entrypoint]
pub struct Trigonometry<T> {
    phantom: PhantomData<T>,
}

}

impl<T: TrignometryParams,U256: std::ops::Shr<i32> + std::ops::Sub<<U256 as std::ops::Shr<i32>>::Output>,U256> Trigonometry<T> {
    pub fn sin(&mut self, _angle: U256) -> Result<U256,Vec<u8>> {
        let _angle = (self.ANGLES_IN_CYCLE * (_angle % self.TWO_PI)) / self.TWO_PI;

        let interp = (_angle >> self.INTERP_OFFSET) & ((1 << self.INTERP_WIDTH) - 1);
        let index = (_angle >> self.INDEX_OFFSET) & ((1 << self.INDEX_WIDTH) - 1);

        let is_odd_quadrant = (_angle & self.QUADRANT_LOW_MASK) == 0;
        let is_negative_quadrant = (_angle & self.QUADRANT_HIGH_MASK) != 0;

        if !is_odd_quadrant {
            index = self.SINE_TABLE_SIZE - U256::from(1) - index;
        }

        let table: &[U8] = &sin_table;

        let offset1_2 = (index + U256::from(2)) * self.Entry_bytes;

        let x1_2: U256;
        unsafe {
            x1_2 = *(table.as_ptr().add(offset1_2) as *const U256);
        }

        let x1 = (x1_2 >> (8 * self.Entry_bytes)) & self.Entry_mask;

        let x2 = x1_2 & self.Entry_mask;

        let approximation = ((x2 - x1) * interp) >> self.INTERP_WIDTH;
        let &mut sine = if is_odd_quadrant {
            I256*(x1) + I256*(approximation)
        } else {
            I256*(x2) - I256*(approximation)
        };
        // if is_negative_quadrant {
        //     sine = sine * U256::from(1);
        //     sine = -1sine
        // }

            
        Ok((sine * U256::from(1_000_000_000_000_000_000)) / U256::from(2_147_483_647))
    }

    pub fn cos(&mut self, angle: U256) -> Result<U256,Vec<u8>> {
        Ok(self.sin(angle + self.PI_OVER_TWO))
    }
}
