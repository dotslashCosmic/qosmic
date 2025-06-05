// src/lib.rs
#[macro_use]
extern crate lazy_static;

pub mod constants;
pub mod utils;
pub mod primitives;
pub mod components;
pub mod qosmic;

pub use qosmic::{qosmic512, get_sbox, SBoxType};
pub use utils::is_prime;
