// src/constants.rs
use lazy_static::lazy_static;
use num_bigint::BigUint;
use num_traits::Num;

lazy_static! {
    pub static ref KEY: BigUint = BigUint::from_str_radix("FC07FC3F8F10ED63B9791090508526A49678B884D7EA514C2D377E4934F78E334CD2FE213CC22B97113A8AE729DE53B2EF4E6C41FC20D46A724E70738BD200A7", 16).unwrap();}
pub const COEFFS: [u64; 5] = [17, 23, 37, 41, 47];
pub const MAGIC: u64 = 0x517CC1B727220A97;
pub const RATIO: u64 = 0x9E3779B97F4A7C15;
pub const CONST: u64 = 0x8F983AA82D9FBEDF;
pub const ARX_BITS: [u32; 8] = [5, 11, 23, 31, 35, 43, 51, 63];
pub const MASK_32: u64 = 0xFFFFFFFF;
pub const MASK_64: u64 = 0xFFFFFFFFFFFFFFFF;
pub const MASK_128: u128 = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
pub const DIM: usize = 1024;
pub const Q_MOD: u64 = 282_429_536_481;
pub const MAGNITUDE: i64 = 6;
