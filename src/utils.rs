// src/utils.rs
use crate::constants::QONST;
use num_bigint::BigUint;
use num_traits::Zero;
use rand::Rng;

pub fn bytes_to_binary_string(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{:08b}", byte)).collect()}

pub fn bit_diff(a: &[u8], b: &[u8]) -> f64 {
    let a_bits = bytes_to_binary_string(a);
    let b_bits = bytes_to_binary_string(b);
    let max_len = a_bits.len().max(b_bits.len());
    let a_padded = format!("{:0>width$}", a_bits, width = max_len);
    let b_padded = format!("{:0>width$}", b_bits, width = max_len);
    let diffs = a_padded
        .chars()
        .zip(b_padded.chars())
        .filter(|(x, y)| x != y)
        .count();
    if a_padded.is_empty() { return 0.0; }
    (diffs as f64 / a_padded.len() as f64) * 100.0}

pub fn modify_input_bytes(input_data: &[u8]) -> Vec<u8> {
    if input_data.is_empty() {
        return Vec::new();}
    let mut modified = input_data.to_vec();
    let last_idx = modified.len() - 1;
    modified[last_idx] ^= 0x01;
    modified}

pub fn modify_input_u64(input_data: u64) -> u64 {
    input_data ^ 0x01}

pub fn derive_deterministic_nonce(data: &[u8]) -> u64 {
    if data.is_empty() {
        return 0;}
    let mut nonce: u64 = 0;
    for chunk in data.chunks(8) {
        let mut block_val: u64 = 0;
        for (i, &byte) in chunk.iter().enumerate() {
            block_val |= (byte as u64) << (8 * (7 - i));}
        nonce = nonce.wrapping_add(block_val);
        nonce ^= nonce.rotate_left(13);}
    nonce}

pub fn is_prime(n: u64) -> bool {
    if n < 2 { return false; }
    const SMALL_PRIMES_LIST: [u64; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    for &p in &SMALL_PRIMES_LIST {
        if n % p == 0 { return n == p; }}
    if n < SMALL_PRIMES_LIST.last().unwrap() * SMALL_PRIMES_LIST.last().unwrap() {
        return true;}
    let mut d = n - 1;
    let mut s = 0;
    while d % 2 == 0 {
        d /= 2;
        s += 1;}
    let mut rng = rand::thread_rng();
    for _ in 0..24 {
        let a = rng.gen_range(2..(n - 2).max(3));
        let mut x = pow_mod(a, d, n);
        if x == 1 || x == n - 1 {
            continue;}
        let mut continue_outer = false;
        for _ in 0..(s - 1) {
            x = pow_mod(x, 2, n);
            if x == n - 1 {
                continue_outer = true;
                break;}}
        if continue_outer {
            continue;}
        return false;}
    true}

pub fn pow_mod(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    if modulus == 0 { panic!("Modulus cannot be zero"); }
    if modulus == 1 { return 0; }
    let mut res = 1;
    base %= modulus;
    while exp > 0 {
        if exp % 2 == 1 {
            res = ((res as u128 * base as u128) % modulus as u128) as u64;}
        base = ((base as u128 * base as u128) % modulus as u128) as u64;
        exp /= 2;}
    res}

pub fn pow_mod_biguint(base: BigUint, exp: BigUint, modulus: BigUint) -> BigUint {
    if modulus.is_zero() { panic!("Modulus is zero in pow_mod_biguint"); }
    base.modpow(&exp, &modulus)}

pub fn bits_vec_to_u8(bits: &[u8]) -> u8 {
    let mut val = 0u8;
    for (idx, &bit) in bits.iter().take(8).enumerate() {
        val |= bit << (7 - idx);}
    val}

pub fn bits_vec_to_u16(bits: &[u8]) -> u16 {
    let mut val = 0u16;
    for (idx, &bit) in bits.iter().take(16).enumerate() {
        val |= (bit as u16) << (15 - idx);}
    val}

pub fn key_as_u512() -> [u128; 4] {
    let mut limbs = QONST.iter_u64_digits();
    let mut u64_limbs: [u64; 8] = [0; 8];
    for i in 0..8 {
        u64_limbs[i] = limbs.next().unwrap_or(0);}
    [((u64_limbs[1] as u128) << 64) | (u64_limbs[0] as u128),
     ((u64_limbs[3] as u128) << 64) | (u64_limbs[2] as u128),
     ((u64_limbs[5] as u128) << 64) | (u64_limbs[4] as u128),
     ((u64_limbs[7] as u128) << 64) | (u64_limbs[6] as u128),]}

pub fn key_as_u128() -> u128 {
    let mut limbs = QONST.iter_u64_digits();
    let lsb = limbs.next().unwrap_or(0);
    let msb = limbs.next().unwrap_or(0);
    ((msb as u128) << 64) | (lsb as u128)}

pub fn key_as_u64() -> u64 {
    QONST.iter_u64_digits().next().unwrap_or(0)}

pub fn key_as_u32() -> u32 {
    key_as_u64() as u32}
