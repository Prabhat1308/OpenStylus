#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use core::{borrow::BorrowMut, market::PhantomData};
use stylus_sdk::sol_storage::StorageSignedU256;
use stylus_sdk::sol_storage::StorageU256;
use stylus_sdk::sol_storage::StorageVec;
use stylus_sdk::{alloy_primitives::U256, prelude::*};

sol_storage!{

    const MIN_64x64 : U256 = -0x80000000000000000000000000000000;
    const MAX_64x64 : U256 = 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
    
}


impl ABDK {

    pub fn from_int(&mut self,&mut x: I256) -> Result<i128> {
        assert!(x >= I256::from(MIN_64x64) && x <= I256::from(MAX_64x64));
        (x << 64) as I128
    }
    
    pub fn to_int(&mut self ,&mut x: I128) -> Result<i64> {
        (x >> 64) as I64
    }

    pub fn from_uint(&mut self ,&mut x: U256) -> Result<i128> {
        assert!(x <= U256::from(MAX_64x64));
        I128::from(I256::from(x << 64))
    }

    pub fn to_uint(&mut self ,&mut x: I128) -> Result<u64> {
        assert!(x >= 0);
        U64::from(U128::from(x >> 64))
    }

    pub fn from_128x128(&mut self ,&mut x: I256) -> Result<i128> {
        let result = x >> 64;
        assert!(result >= I256::from(MIN_64x64) && result <= I256::from(MAX_64x64));
        result as I128
    }

    pub fn to_128x128(&mut self,&mut x: I128) -> Result<i256> {
        (x as i256) << 64
    }

    pub fn add(&mut self,&mut x: I128,&mut y: I128) -> Result<i128> {
        let result = I256::from(x) + I256::from(y);
        assert!(result >= I256::from(MIN_64x64) && result <= I256::from(MAX_64x64));
        result as I128
    }

    pub fn sub(&mut self ,&mut x: I128,&mut y: I128) -> Result<i128> {
        let result = I256::from(x) - I256::from(y);
        assert!(result >= I256::from(MIN_64x64) && result <= I256::from(MAX_64x64));
        result as I128
    }

    pub fn mul(&mut self ,&mut x: i128, &mut y: i128) -> Result<i128> {
        let result = (I256::from(x) * I256::from(y)) >> 64;
        assert!(result >= I256::from(MIN_64x64) && result <= I256::from(MAX_64x64));
        result as I128
    }

    pub fn muli(&mut self ,&mut x: I128,&mut  y: I256) -> Result<i256> {
        if x == MIN_64x64 {
            assert!(y >= I256::from(-0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF) &&
                    y <= I256::from(0x1000000000000000000000000000000000000000000000000));
            return -(y << 63);
        } else {
            let mut negative_result = false;
            let mut x = x;
            let mut y = y;
            if x < 0 {
                x = -x;
                negative_result = true;
            }
            if y < 0 {
                y = -y; // We rely on overflow behavior here
                negative_result = !negative_result;
            }
            let absolute_result = mulu(x, y.into());
            if negative_result {
                assert!(absolute_result <=
                        U256::from(0x8000000000000000000000000000000000000000000000000000000000000000));
                return -(absolute_result.into()); // We rely on overflow behavior here
            } else {
                assert!(absolute_result <=
                        U256::from(0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));
                return absolute_result.into();
            }
        }
    }

    pub fn mulu(&mut self,&mut x: I128, &mut y: U256) -> Result<u256> {
        if y == 0 {
            return 0.into();
        }
    
        assert!(x >= 0);
    
        let lo = (U256::from(x as i256) * (y & U256::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF))) >> 64;
        let hi = U256::from(x as i256) * (y >> 128);
    
        assert!(hi <= U256::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));
        let mut hi = hi << 64;
    
        assert!(hi <= U256::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF - lo));
        hi + lo
    }

    pub fn div(&mut self,&mut x: I128,&mut y: I128) -> Result<i128> {
        assert!(y != 0);
        let result = (I256::from(x) << 64) / y;
        assert!(result >= I256::from(MIN_64x64) && result <= I256::from(MAX_64x64));
        result as I128
    }

    pub fn divi(&mut self ,&mut x: I256,&mut y: I256) -> Result<i128> {
        assert!(y != 0);
    
        let mut negative_result = false;
        let mut x = x;
        let mut y = y;
        if x < 0 {
            x = -x; // We rely on overflow behavior here
            negative_result = true;
        }
        if y < 0 {
            y = -y; // We rely on overflow behavior here
            negative_result = !negative_result;
        }
        let absolute_result = divuu(x.into(), y.into());
        if negative_result {
            assert!(absolute_result <= 0x80000000000000000000000000000000);
            return -(absolute_result as I128); // We rely on overflow behavior here
        } else {
            assert!(absolute_result <= 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
            return absolute_result as I128; // We rely on overflow behavior here
        }
    }

    pub fn divu(&mut self , &mut x: U256,&mut y: U256) -> Result<i128> {
        assert!(y != 0);
        let result = divuu(x, y);
        assert!(result <= U128::from(MAX_64x64));
        result as I128
    }

    pub fn neg(&mut self ,&mut x: I128) -> Result<i128> {
        assert!(x != MIN_64x64);
        -x
    }

    pub fn ABS (&mut self ,&mut x: I128) -> Result<i128> {
        assert!(x != MIN_64x64);
        if x < 0 {
            -x
        } else {
            x
        }
    }
    
    pub fn inv(&mut self ,&mut x: I128) -> Result<i128> {
        assert!(x != 0);
        let result = I256::from(0x100000000000000000000000000000000) / x;
        assert!(result >= I256::from(MIN_64x64) && result <= I256::from(MAX_64x64));
        result as I128
    }

    pub fn avg(&mut self ,&mut x: I128,&mut y: I128) -> Result<i128> {
        (I256::from(x) + I256::from(y)) >> 1
    }
    
    pub fn gavg(&mut self ,&mut x: I128,&mut y: I128) -> Result<i128> {
        let m = I256::from(x) * I256::from(y);
        assert!(m >= 0);
        assert!(m < 0x4000000000000000000000000000000000000000000000000000000000000000);
        sqrtu(m.into()) as I128
    }

    pub fn pow(&mut self ,&mut x: I128,&mut y: U256) -> Result<i128> {
        let negative = x < 0 && y & 1 == 1;
    
        let abs_x = if x < 0 { -x } else { x } as U256;
        let mut abs_result: U256 = 0x100000000000000000000000000000000.into();
    
        if abs_x <= 0x10000000000000000.into() {
            let mut abs_x_shift: U256 = 63.into();
            if abs_x < 0x1000000000000000000000000.into() { abs_x <<= 32; abs_x_shift -= 32.into(); }
            if abs_x < 0x10000000000000000000000000000.into() { abs_x <<= 16; abs_x_shift -= 16.into(); }
            if abs_x < 0x1000000000000000000000000000000.into() { abs_x <<= 8; abs_x_shift -= 8.into(); }
            if abs_x < 0x10000000000000000000000000000000.into() { abs_x <<= 4; abs_x_shift -= 4.into(); }
            if abs_x < 0x40000000000000000000000000000000.into() { abs_x <<= 2; abs_x_shift -= 2.into(); }
            if abs_x < 0x80000000000000000000000000000000.into() { abs_x <<= 1; abs_x_shift -= 1.into(); }
    
            let mut result_shift: U256 = 0.into();
            while y != 0 {
                assert!(abs_x_shift < 64.into());
    
                if y & 0x1 != 0 {
                    abs_result = abs_result * abs_x >> 127;
                    result_shift += abs_x_shift;
                    if abs_result > 0x100000000000000000000000000000000.into() {
                        abs_result >>= 1;
                        result_shift += 1.into();
                    }
                }
                abs_x = abs_x * abs_x >> 127;
                abs_x_shift <<= 1;
                if abs_x >= 0x100000000000000000000000000000000.into() {
                    abs_x >>= 1;
                    abs_x_shift += 1.into();
                }
    
                y >>= 1;
            }
    
            assert!(result_shift < 64.into());
            abs_result >>= 64 - result_shift;
        } else {
            panic!("Unsupported value of abs_x");
        }
    
        let result = if negative { -(abs_result as I256) } else { abs_result as I256 };
        assert!(result >= MIN_64x64 && result <= MAX_64x64);
        result as I128
    }

    pub fn sqrt(&mut self ,&mut x: I128) -> Result<i128> {
        assert!(x >= 0);
        sqrtu((x as u256).into()) as I128
    }

    pub fn log_2(&mut self,&mut x: I128) -> Result<i128> {
        assert!(x > 0);
    
        let mut msb: i256 = 0.into();
        let mut xc: i256 = x.into();
        if xc >= 0x10000000000000000.into() { xc >>= 64; msb += 64.into(); }
        if xc >= 0x100000000.into() { xc >>= 32; msb += 32.into(); }
        if xc >= 0x10000.into() { xc >>= 16; msb += 16.into(); }
        if xc >= 0x100.into() { xc >>= 8; msb += 8.into(); }
        if xc >= 0x10.into() { xc >>= 4; msb += 4.into(); }
        if xc >= 0x4.into() { xc >>= 2; msb += 2.into(); }
        if xc >= 0x2.into() { msb += 1.into(); }  // No need to shift xc anymore
    
        let mut result: I256 = (msb - 64).into() << 64;
        let mut ux: U256 = (x as I256).into() << (127 - msb).into();
        let mut bit: I256 = 0x8000000000000000.into();
        while bit > 0 {
            ux *= ux;
            let b: U256 = ux >> 255;
            ux >>= 127 + b;
            result += bit * (b as I256);
            bit >>= 1;
        }
    
        result as I128
    }

    pub fn ln(&mut self , &mut x: I128) -> Result<i128> {
        assert!(x > 0);
    
        (log_2(x) as U256 * 0xB17217F7D1CF79ABC9E3B39803F2F6AF >> 128) as U256 as I128
    }

    pub fn exp_2(&mut self , &mut x: I128) -> Result<i128> {
        const MAX_64x64: I128 = 0x7FFFFFFFFFFFFFFF;
        let mut result: U128 = 0x80000000000000000000000000000000;
    
        if x & 0x8000000000000000 > 0 {
            result = result.wrapping_mul(0x16A09E667F3BCC908B2FB1366EA957D3E) >> 128;
        }
        if x & 0x4000000000000000 > 0 {
            result = result.wrapping_mul(0x1306FE0A31B7152DE8D5A46305C85EDEC) >> 128;
        }

    
        result >>= (63 - (x >> 64)) as U128;
        assert!(result <= MAX_64x64 as U128);
    
        result as I128
    }

    pub fn exp_2(&mut self , &mut x: I128) -> Result<i128> {
        const MAX_64x64: I128 = 0x7FFFFFFFFFFFFFFF;
        let mut result: U128 = 0x80000000000000000000000000000000;
    
        if x & 0x8000000000000000 > 0 {
            result = result.wrapping_mul(0x16A09E667F3BCC908B2FB1366EA957D3E) >> 128;
        }
        if x & 0x4000000000000000 > 0 {
            result = result.wrapping_mul(0x1306FE0A31B7152DE8D5A46305C85EDEC) >> 128;
        }
        if x & 0x2000000000000000 > 0 {
            result = result.wrapping_mul(0x1172B83C7D517ADCDF7C8C50EB14A791F) >> 128;
        }
        if x & 0x1000000000000000 > 0 {
            result = result.wrapping_mul(0x10B5586CF9890F6298B92B71842A98363) >> 128;
        }
        if x & 0x800000000000000 > 0 {
            result = result.wrapping_mul(0x1059B0D31585743AE7C548EB68CA417FD) >> 128;
        }
        if x & 0x400000000000000 > 0 {
            result = result.wrapping_mul(0x102C9A3E778060EE6F7CACA4F7A29BDE8) >> 128;
        }
        if x & 0x200000000000000 > 0 {
            result = result.wrapping_mul(0x10163DA9FB33356D84A66AE336DCDFA3F) >> 128;
        }
        if x & 0x100000000000000 > 0 {
            result = result.wrapping_mul(0x100B1AFA5ABCBED6129AB13EC11DC9543) >> 128;
        }
        if x & 0x80000000000000 > 0 {
            result = result.wrapping_mul(0x10058C86DA1C09EA1FF19D294CF2F679B) >> 128;
        }
        if x & 0x40000000000000 > 0 {
            result = result.wrapping_mul(0x1002C605E2E8CEC506D21BFC89A23A00F) >> 128;
        }
        if x & 0x20000000000000 > 0 {
            result = result.wrapping_mul(0x100162F3904051FA128BCA9C55C31E5DF) >> 128;
        }
        if x & 0x10000000000000 > 0 {
            result = result.wrapping_mul(0x1000B175EFFDC76BA38E31671CA939725) >> 128;
        }
        if x & 0x8000000000000 > 0 {
            result = result.wrapping_mul(0x100058BA01FB9F96D6CACD4B180917C3D) >> 128;
        }
        if x & 0x4000000000000 > 0 {
            result = result.wrapping_mul(0x10002C5CC37DA9491D0985C348C68E7B3) >> 128;
        }
        if x & 0x2000000000000 > 0 {
            result = result.wrapping_mul(0x1000162E525EE054754457D5995292026) >> 128;
        }
        if x & 0x1000000000000 > 0 {
            result = result.wrapping_mul(0x10000B17255775C040618BF4A4ADE83FC) >> 128;
        }
        if x & 0x800000000000 > 0 {
            result = result.wrapping_mul(0x1000058B91B5BC9AE2EED81E9B7D4CFAB) >> 128;
        }
        if x & 0x400000000000 > 0 {
            result = result.wrapping_mul(0x100002C5C89D5EC6CA4D7C8ACC017B7C9) >> 128;
        }
        if x & 0x200000000000 > 0 {
            result = result.wrapping_mul(0x10000162E43F4F831060E02D839A9D16D) >> 128;
        }
        if x & 0x100000000000 > 0 {
            result = result.wrapping_mul(0x100000B1721BCFC99D9F890EA06911763) >> 128;
        }
        if x & 0x80000000000 > 0 {
            result = result.wrapping_mul(0x10000058B90CF1E6D97F9CA14DBCC1628) >> 128;
        }
        if x & 0x40000000000 > 0 {
            result = result.wrapping_mul(0x1000002C5C863B73F016468F6BAC5CA2B) >> 128;
        }
        if x & 0x20000000000 > 0 {
            result = result.wrapping_mul(0x100000162E430E5A18F6119E3C02282A5) >> 128;
        }
        if x & 0x10000000000 > 0 {
            result = result.wrapping_mul(0x1000000B1721835514B86E6D96EFD1BFE) >> 128;
        }
        if x & 0x8000000000 > 0 {
            result = result.wrapping_mul(0x100000058B90C0B48C6BE5DF846C5B2EF) >> 128;
        }
        if x & 0x4000000000 > 0 {
            result = result.wrapping_mul(0x10000002C5C8601CC6B9E94213C72737A) >> 128;
        }
        if x & 0x2000000000 > 0 {
            result = result.wrapping_mul(0x1000000162E42FFF037DF38AA2B219F06) >> 128;
        }
        if x & 0x1000000000 > 0 {
            result = result.wrapping_mul(0x10000000B17217FBA9C739AA5819F44F9) >> 128;
        }
        if x & 0x800000000 > 0 {
            result = result.wrapping_mul(0x1000000058B90BFCDEE5ACD3C1CEDC823) >> 128;
        }
        if x & 0x400000000 > 0 {
            result = result.wrapping_mul(0x100000002C5C85FE31F35A6A30DA1BE50) >> 128;
        }
        if x & 0x200000000 > 0 {
            result = result.wrapping_mul(0x10000000162E42FF0999CE3541B9FFFCF) >> 128;
        }
        if x & 0x100000000 > 0 {
            result = result.wrapping_mul(0x100000000B17217F80F4EF5AADDA45554) >> 128;
        }
        if x & 0x80000000 > 0 {
            result = result.wrapping_mul(0x10000000058B90BFBF8479BD5A81B51AD) >> 128;
        }
        if x & 0x40000000 > 0 {
            result = result.wrapping_mul(0x1000000002C5C85FDF84BD62AE30A74CC) >> 128;
        }
        if x & 0x20000000 > 0 {
            result = result.wrapping_mul(0x100000000162E42FEFB2FED257559BDAA) >> 128;
        }
        if x & 0x10000000 > 0 {
            result = result.wrapping_mul(0x1000000000B17217F7D5A7716BBA4A9AE) >> 128;
        }
        if x & 0x8000000 > 0 {
            result = result.wrapping_mul(0x100000000058B90BFBE9DDBAC5E109CCE) >> 128;
        }
        if x & 0x4000000 > 0 {
            result = result.wrapping_mul(0x10000000002C5C85FDF4B15DE6F17EB0D) >> 128;
        }
        if x & 0x2000000 > 0 {
            result = result.wrapping_mul(0x1000000000162E42FEFA494F1478FDE05) >> 128;
        }
        if x & 0x1000000 > 0 {
            result = result.wrapping_mul(0x10000000000B17217F7D20CF927C8E94C) >> 128;
        }
        if x & 0x800000 > 0 {
            result = result.wrapping_mul(0x1000000000058B90BFBE8F71CB4E4B33D) >> 128;
        }
        if x & 0x400000 > 0 {
            result = result.wrapping_mul(0x100000000002C5C85FDF477B662B26945) >> 128;
        }
        if x & 0x200000 > 0 {
            result = result.wrapping_mul(0x10000000000162E42FEFA3AE53369388C) >> 128;
        }
        if x & 0x100000 > 0 {
            result = result.wrapping_mul(0x100000000000B17217F7D1D351A389D40) >> 128;
        }
        if x & 0x80000 > 0 {
            result = result.wrapping_mul(0x10000000000058B90BFBE8E8B2D3D4EDE) >> 128;
        }
        if x & 0x40000 > 0 {
            result = result.wrapping_mul(0x1000000000002C5C85FDF4741BEA6E77E) >> 128;
        }
        if x & 0x20000 > 0 {
            result = result.wrapping_mul(0x100000000000162E42FEFA39FE95583C2) >> 128;
        }
        if x & 0x10000 > 0 {
            result = result.wrapping_mul(0x1000000000000B17217F7D1CFB72B45E1) >> 128;
        }
        if x & 0x8000 > 0 {
            result = result.wrapping_mul(0x100000000000058B90BFBE8E7CC35C3F0) >> 128;
        }
        if x & 0x4000 > 0 {
            result = result.wrapping_mul(0x10000000000002C5C85FDF473E242EA38) >> 128;
        }
        if x & 0x2000 > 0 {
            result = result.wrapping_mul(0x1000000000000162E42FEFA39F02B772C) >> 128;
        }
        if x & 0x1000 > 0 {
            result = result.wrapping_mul(0x10000000000000B17217F7D1CF7D83C1A) >> 128;
        }
        if x & 0x800 > 0 {
            result = result.wrapping_mul(0x1000000000000058B90BFBE8E7BDCBE2E) >> 128;
        }
        if x & 0x400 > 0 {
            result = result.wrapping_mul(0x100000000000002C5C85FDF473DEA871F) >> 128;
        }
        if x & 0x200 > 0 {
            result = result.wrapping_mul(0x10000000000000162E42FEFA39EF44D91) >> 128;
        }
        if x & 0x100 > 0 {
            result = result.wrapping_mul(0x100000000000000B17217F7D1CF79ABCA) >> 128;
        }
        if x & 0x80 > 0 {
            result = result.wrapping_mul(0x10000000000000058B90BFBE8E7BCE544) >> 128;
        }
        if x & 0x40 > 0 {
            result = result.wrapping_mul(0x1000000000000002C5C85FDF473DE6ECA) >> 128;
        }
        if x & 0x20 > 0 {
            result = result.wrapping_mul(0x100000000000000162E42FEFA39EF358D) >> 128;
        }
        if x & 0x10 > 0 {
            result = result.wrapping_mul(0x1000000000000000B17217F7D1CF79ABC) >> 128;
        }
        if x & 0x8 > 0 {
            result = result.wrapping_mul(0x100000000000000058B90BFBE8E7BCD6D) >> 128;
        }
        if x & 0x4 > 0 {
            result = result.wrapping_mul(0x10000000000000002C5C85FDF473DE6B2) >> 128;
        }
        if x & 0x2 > 0 {
            result = result.wrapping_mul(0x1000000000000000162E42FEFA39EF34E) >> 128;
        }
        if x & 0x1 > 0 {
            result = result.wrapping_mul(0x10000000000000000B17217F7D1CF79AB) >> 128;
        }
    
        result >>= (63 - (x >> 64)) as U128;
        assert!(result <= MAX_64x64 as U128);
    
        result as I128
    }

    pub fn exp(&mut self , &mut x: I128) -> Result<i128> {
        assert!(x < 0x400000000000000000); // Overflow
    
        if x < -0x400000000000000000 { return 0; } // Underflow
    
        exp_2((x as I256 * 0x171547652B82FE1777D0FFDA0D23A7D12 >> 128) as I128)
    }

    pub fn divuu(&mut self ,&mut x: U256,&mut  y: U256) -> Result<u128> {
        assert!(y != 0);
    
        let mut result: U256;
    
        if x <= 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF {
            result = (x << 64) / y;
        } else {
            let mut msb: U256 = 192.into();
            let mut xc: U256 = x >> 192;
            if xc >= 0x100000000 { xc >>= 32; msb += 32.into(); }
            if xc >= 0x10000 { xc >>= 16; msb += 16.into(); }
            if xc >= 0x100 { xc >>= 8; msb += 8.into(); }
            if xc >= 0x10 { xc >>= 4; msb += 4.into(); }
            if xc >= 0x4 { xc >>= 2; msb += 2.into(); }
            if xc >= 0x2 { msb += 1.into(); }  // No need to shift xc anymore
    
            result = (x << (255 - msb)) / ((y - 1 >> (msb - 191)) + 1);
            assert!(result <= 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    
            let hi = result * (y >> 128);
            let lo = result * (y & 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    
            let xh = x >> 192;
            let xl = x << 64;
    
            if xl < lo { xh -= 1; }
            xl -= lo; // We rely on overflow behavior here
            lo = hi << 128;
            if xl < lo { xh -= 1; }
            xl -= lo; // We rely on overflow behavior here
    
            result += if xh == hi >> 128 { xl / y } else { 1 };
        }
    
        assert!(result <= 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
        result as U128
    }

    pub fn sqrtu(&mut self , &mut x: U256) -> Result<u128> {
        if x == 0 { return 0; }
        else {
            let mut xx = x;
            let mut r: u256 = 1.into();
            if xx >= 0x100000000000000000000000000000000 { xx >>= 128; r <<= 64; }
            if xx >= 0x10000000000000000 { xx >>= 64; r <<= 32; }
            if xx >= 0x100000000 { xx >>= 32; r <<= 16; }
            if xx >= 0x10000 { xx >>= 16; r <<= 8; }
            if xx >= 0x100 { xx >>= 8; r <<= 4; }
            if xx >= 0x10 { xx >>= 4; r <<= 2; }
            if xx >= 0x4 { r <<= 1; }
            r = (r + x / r) >> 1;
            r = (r + x / r) >> 1;
            r = (r + x / r) >> 1;
            r = (r + x / r) >> 1;
            r = (r + x / r) >> 1;
            r = (r + x / r) >> 1;
            r = (r + x / r) >> 1; // Seven iterations should be enough
            let r1 = x / r;
            return if r < r1 { r as U128 } else { r1 as U128 };
        }
    }



}

