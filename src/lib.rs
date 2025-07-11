// src/lib.rs
#[macro_use]
extern crate lazy_static;

pub mod constants;
pub mod utils;
pub mod primitives;
pub mod components;
pub mod core;
pub mod encode;

pub use core::{get_sbox, hmac_qosmic, qosmic_unkeyed, SBoxType, hash_password, pbkdf2_hmac_qosmic};
pub use utils::is_prime;
