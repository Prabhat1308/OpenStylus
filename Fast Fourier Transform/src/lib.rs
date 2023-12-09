#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use crate::trig::{TrignometryParams, Trigonometry};
use stylus_sdk::sol_storage::{StorageSignedU256,StorageU256,StorageVec};
use stylus_sdk::{alloy_primitives::U256, prelude::*};
use crate::market::PhantomData;
mod trig;

pub trait FourierParams {

}

impl TrignometryParams for FourierParams {
    const INDEX_WIDTH: U256 = U256::from(8);
    const INTERP_WIDTH: U256 = U256::from(8);
    const INDEX_OFFSET: U256 = U256::from(28) - Self::INDEX_WIDTH;
    const INTERP_OFFSET: U256 = Self::INDEX_OFFSET - Self::INTERP_WIDTH;
    const QUADRANT_HIGH_MASK: U256 = U256::from(536_870_912);
    const QUADRANT_LOW_MASK: U256 = U256::from(268_435_456);
    const SINE_TABLE_SIZE: U256 = U256::from(256);
    const PI: U256 = U256::from(3_141_592_653_589_793_238);
    const TWO_PI: U256 = U256::from(2) * Self::PI;
    const PI_OVER_TWO: U256 = Self::PI / U256::from(2);
    const ENTRY_BYTES: U256 = U256::from(4);
    const ENTRY_MASK: U256 = U256::from(1) << (8 * Self::ENTRY_BYTES) - 1;
    
    const SIN_TABLE_: &'static [u8] = &[
        0x00, 0x00, 0x00, 0x00, 0xc9, 0x0f, 0xda, 0xa2, 0x43, 0x3c, 0x5b, 0x26, 0x1d, 0x7f, 0x36,
        0x09, 0x0f, 0x34, 0x7e, 0x9c, 0x3f, 0x95, 0x0f, 0xdc, 0x26, 0x7a, 0x9c, 0xa4, 0x3d, 0x3f,
        0x84, 0x5b, 0x4b, 0x77, 0x3f, 0x0a, 0x3d, 0x70, 0x26, 0x3f, 0x5b, 0x2d, 0x7a, 0x3f, 0x0a,
        0x3d, 0x70, 0x26, 0x3f, 0x5b, 0x2d, 0x7a, 0x3f, 0x0a, 0x3d, 0x70, 0x26, 0x3f, 0x5b, 0x2d,
        0x7a, 0x3f, 0x0a, 0x3d, 0x70, 0x26, 0x3f, 0x5b, 0x2d, 0x7a, 0x3f, 0x0a, 0x3d, 0x70, 0x26,
        0x3f, 0x5b, 0x2d, 0x7a, 0x3f, 0x0a, 0x3d, 0x70, 0x26, 0x3f];
}

sol_storage! {

   #[entrypoint]
   pub struct  Complex {
    Uint256 real;
    Uint256 imag;
   }

    pub struct pi {
        const PI: U256 = U256::from(3141592653589793238);
    }

    struct Fourier {
        #[borrow]
        Trigonometry<FourierParams>trig;
    }
}

#[external]

impl Fourier {
    pub fn log2(&mut self, num: U256) -> Result<u256,Vec<u8>> {
        let log_val: U256 = U256::from(0);

        while num > U256::from(1) {
            num = num / U256::from(2);
            log_val = log_val + U256::from(1);
        }

        return log_val;
    }

    #[view]
    pub fn fft(
         self,
        real_part: &mut StorageVec<StorageSignedU256>,
        img_part: &mut StorageVec<StorageSignedU256>,
    ) -> Result<Vec<i256>, Vec<i256>> {
        let N: U256 = real_part.len();
        let mut k: U256 = N;
        let mut n: U256;
        let thetaT: U256 = self.PI / N; 

        let mut phiT: Complex = Complex::new(Trigonometry::cos(thetaT), Trigonometry::sin(thetaT));

        let T: Complex;

        while k > 1 {
            n = k;
            k >>= 1;
            let phiT_real_temp = phiT.real;
            phiT.real = ((phiT.real * phiT.real) / U256::from(1_000_000_000_000_000_000)) - ((phiT.img * phiT.img) / U256::from(1_000_000_000_000_000_000));
            phiT.img = (U256::from(2) * phiT.img * phiT_real_temp) / U256::from(1_000_000_000_000_000_000);

            T.real = U256::from(1) * U256::from(1_000_000_000_000_000_000);
            T.img = U256::from(0) * U256::from(1_000_000_000_000_000_000);

            for l in 0..k {
                for a in (l..N.as_usize()).step_by(n) {
                    let b = a + k;

                    let mut t = Complex {
                        real: self.real_part[a] - self.real_part[b],
                        img: complex_part[a] - complex_part[b],
                    };
                    real_part[a] = real_part[a] + real_part[b];
                    complex_part[a] = complex_part[a] + complex_part[b];
                    real_part[b] = ((T.real * t.real) /U256::from(1_000_000_000_000_000_000)) - ((T.img * t.img) /U256::from(1_000_000_000_000_000_000));
                    complex_part[b] = ((T.real * t.img) /U256::from(1_000_000_000_000_000_000)) + ((T.img * t.real) /U256::from(1_000_000_000_000_000_000));
                }
                let T_real_temp = T.real;
                T.real = ((T.real * phiT.real) /U256::from(1_000_000_000_000_000_000)) - ((T.img * phiT.img) / U256::from(1_000_000_000_000_000_000));
                T.img = ((T_real_temp * phiT.img) /U256::from(1_000_000_000_000_000_000)) + ((T.img * phiT.real) /U256::from(1_000_000_000_000_000_000));
            }
        }

        let m = self.log2(n);

        for a in 0..N {
            let mut b = U256::from(a);
            b = ((b & U256::from(0xaaaaaaaa)) >> 1) | ((b &U256::from(0x55555555) ) << 1);
            b = ((b & U256::from(0xcccccccc)) >> 2) | ((b & U256::from(0x33333333))) << 2;
            b = (((b & U256::from(0xf0f0f0f0))) >> 4) | ((b & U256::from(0x0f0f0f0f)) << 4);
            b = ((b & U256::from(0xff00ff00)) >> 8) | ((b & U256::from(0x00ff00ff)) << 8);
            b = ((b >> 16) | (b << 16)) >> (32 - m);

            if b > U256::from(a) {
                let t_real = real_part[a];
                let t_img = complex_part[a];
                real_part[a] = real_part[b];
                complex_part[a] = complex_part[b];
                real_part[b] = t_real;
                complex_part[b] = t_img;
            }

            (real_part, complex_part)
        }
    }
}
