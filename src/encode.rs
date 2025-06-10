// src/encode.rs
use hex;
use num_bigint::BigUint;
use num_traits::Num;
use base64::{engine::general_purpose, Engine as _};
use bs58;
use base36;

pub fn to_base36(hex_string: &str) -> String {
    let bytes = hex::decode(hex_string).expect("Failed to decode hex string to bytes for Base36 encoding");
    let base36_string = base36::encode(&bytes);
    let padded_length = 100;
    if base36_string.len() < padded_length {
        let mut padded_string = String::with_capacity(padded_length);
        for _ in 0..(padded_length - base36_string.len()) {
            padded_string.push('q');}
        padded_string.push_str(&base36_string);
        padded_string
    } else if base36_string.len() > padded_length {
        base36_string[0..padded_length].to_string()
    } else {
        if base36_string.chars().next() != Some('q') {
            let mut chars: Vec<char> = base36_string.chars().collect();
            chars[0] = 'q';
            chars.into_iter().collect()
        } else {
            base36_string}}}

pub fn to_binary(hex_string: &str) -> String {
    let bytes = hex::decode(hex_string).expect("Failed to decode hex string");
    bytes.iter()
        .map(|&byte| format!("{:08b}", byte))
        .collect()}

pub fn to_base64(hex_string: &str) -> String {
    let bytes = hex::decode(hex_string).expect("Failed to decode hex string to bytes for Base64 encoding");
    general_purpose::STANDARD.encode(&bytes)}

pub fn to_base58(hex_string: &str) -> String {
    let bytes = hex::decode(hex_string).expect("Failed to decode hex string to bytes for Base58 encoding");
    bs58::encode(&bytes).into_string()}
