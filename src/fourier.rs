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
use hex;

mod trig;

impl TrignometryParams for FourierParams {
    const INDEX_WIDTH: u256 = 8;
    const INTERP_WIDTH: u256 = 8;
    const INDEX_OFFSET: u256 = 28 - INDEX_WIDTH;
    const INTERP_OFFSET: u256 = INDEX_OFFSET - INTERP_WIDTH;
    const QUADRANT_HIGH_MASK: U256 = 536_870_912;
    const QUADRANT_LOW_MASK: U256 = 268_435_456;
    const SINE_TABLE_SIZE: U256 = 256;
    const PI: U256 = 3_141_592_653_589_793_238;
    const TWO_PI: U256 = 2 * PI;
    const PI_OVER_TWO: U256 = PI / 2;
    const ENTRY_BYTES: U256 = 4;
    const ENTRY_MASK: U256 = (1 << (8 * ENTRY_BYTES)) - 1;
    
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
    uint256 real;
    uint256 imag;
   }

    pub trait PI {
        const pi : u256 = 3141592653589793238;
    }

    struct Fourier {
        #[borrow]
        Trigonometry<FourierParams>trig;
    }
}

#[external]

impl Fourier {
    pub fn log2(&mut self, num: u256) -> Result<u256> {
        let log_val: u256 = 0;

        while (num > 1) {
            num = num / 2;
            log_val = log_val + 1;
        }

        return log_val;
    }

    #[view]
    pub fn fft(
        &mut self,
        real_part: &mut StorageVec<StorageSignedU256>,
        img_part: &mut StorageVec<StorageSignedU256>,
    ) -> Result<vec<i256>, vec<i256>> {
        let N: u256 = real_part.len();
        let mut k: u256 = N;
        let mut n: u256;
        let thetaT: u256 = PI / N; 

        let mut phiT: Complex = Complex::new(Trigonometry::cos(thetaT), Trigonometry::sin(thetaT));

        let T: Complex;

        while k > 1 {
            n = k;
            k >>= 1;
            let phiT_real_temp = phiT.real;
            phiT.real = (((phiT.real * phiT.real) / 1e18) - ((phiT.img * phiT.img) / 1e18));
            phiT.img = (2 * phiT.img * phiT_real_temp) / 1e18;

            T.real = 1 * 1e18;
            T.img = 0 * 1e18;
            for l in 0..k {
                for a in (l..N).step_by(n) {
                    let b = a + k;

                    let mut t = Complex {
                        real: real_part[a] - real_part[b],
                        img: complex_part[a] - complex_part[b],
                    };
                    real_part[a] = real_part[a] + real_part[b];
                    complex_part[a] = complex_part[a] + complex_part[b];
                    real_part[b] = (((T.real * t.real) / 1e18) - ((T.img * t.img) / 1e18));
                    complex_part[b] = (((T.real * t.img) / 1e18) + ((T.img * t.real) / 1e18));
                }
                let T_real_temp = T.real;
                T.real = (((T.real * phiT.real) / 1e18) - ((T.img * phiT.img) / 1e18));
                T.img = (((T_real_temp * phiT.img) / 1e18) + ((T.img * phiT.real) / 1e18));
            }
        }

        let m = self.log2(n);

        for a in 0..N {
            let mut b = a;
            b = (((b & 0xaaaaaaaa) >> 1) | ((b & 0x55555555) << 1));
            b = (((b & 0xcccccccc) >> 2) | ((b & 0x33333333) << 2));
            b = (((b & 0xf0f0f0f0) >> 4) | ((b & 0x0f0f0f0f) << 4));
            b = (((b & 0xff00ff00) >> 8) | ((b & 0x00ff00ff) << 8));
            b = ((b >> 16) | (b << 16)) >> (32 - m);

            if b > a {
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
