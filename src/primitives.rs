// src/primitives.rs
use crate::constants::*;
use crate::qosmic::SBoxType;
use crate::utils::{key_as_u128, key_as_u64};
use ndarray::{Array1, Array2, Axis};
use num_traits::ToBytes;
use rand::Rng;
use std::convert::TryInto;
use log::debug;

const POLY_GF2_9_DEG9: u16 = 0x211;

pub fn encrypt_internal(message_bits: &Array1<u8>) -> (Array2<u64>, Array1<u64>) {
    let num_bits = message_bits.len();
    let mut rng = rand::thread_rng();
    let mut a_flat = Vec::with_capacity(DIM * num_bits);
    for _ in 0..(DIM * num_bits) {
        a_flat.push(rng.gen_range(0..Q_MOD));}
    let a_list = Array2::from_shape_vec((num_bits, DIM), a_flat).expect("Failed to reshape a_list for LWE matrix A");
    debug!("encrypt_internal: Generated A_list matrix (first 3 rows): {:?}", a_list.rows().into_iter().take(3).collect::<Vec<_>>());
    let mut errors_vec = Vec::with_capacity(num_bits);
    for _ in 0..num_bits {
        errors_vec.push(rng.gen_range(-MAGNITUDE..(MAGNITUDE + 1)));}
    let errors = Array1::from_vec(errors_vec);
    debug!("encrypt_internal: Generated errors vector (first 10 elements): {:?}", &errors.as_slice().unwrap()[..std::cmp::min(errors.len(), 10)]);
    let a_sq = a_list.mapv(|x| (x as u128 * x as u128 % Q_MOD as u128) as u64);
    let sum_a_sq: Array1<u64> = a_sq.sum_axis(Axis(1));
    let message_bits_u64 = message_bits.mapv(|x| x as u64);
    debug!("encrypt_internal: Message bits converted to u64: {:?}", &message_bits_u64.as_slice().unwrap()[..std::cmp::min(message_bits_u64.len(), 10)]);
    let term3_message_encoding = message_bits_u64.mapv(|bit| (Q_MOD / 2) * bit);
    let b_ciphertext = sum_a_sq
        .iter()
        .zip(errors.iter())
        .zip(term3_message_encoding.iter())
        .map(|((&sa_sq_val, &err_val), &term3_val)| {
            let sa_sq_mod_q = sa_sq_val % Q_MOD;
            let term1_plus_2_mod_q = (sa_sq_mod_q as i128 + err_val as i128 + Q_MOD as i128) % Q_MOD as i128;
            let result = (term1_plus_2_mod_q as u128 + term3_val as u128) % Q_MOD as u128;
            result as u64})
        .collect::<Array1<u64>>();
    (a_list, b_ciphertext)}

pub fn quantum_internal(input_data: &[u8]) -> Vec<u8> {
    let mut hash_value = vec![0u8; 128];
    let input_len = input_data.len();
    let block_size: usize = 4096;
    for i in (0..input_len).step_by(block_size) {
        let end = (i + block_size).min(input_len);
        let mut block_bytes = input_data[i..end].to_vec();
        if block_bytes.len() < block_size {
            block_bytes.resize(block_size, 0x00);}
        let mut bits_unpacked = Vec::new();
        for byte_val in &block_bytes {
            for k in 0..8 {
                bits_unpacked.push((byte_val >> (7 - k)) & 1);}}
        bits_unpacked.truncate(DIM);
        let bits_arr = Array1::from_vec(bits_unpacked);
        let (_a_list, b_values) = encrypt_internal(&bits_arr);
        let mut state_u64 = Vec::with_capacity(block_size / 8);
        for chunk in block_bytes.chunks_exact(8) {
            state_u64.push(u64::from_be_bytes(chunk.try_into().unwrap()));}
        let mut state_arr = Array1::from_vec(state_u64);
        let state_arr_len = state_arr.len();
        if state_arr_len > 0 {
            for (idx, &b_val) in b_values.iter().enumerate() {
                let effect = b_val % Q_MOD;
                state_arr[idx % state_arr_len] ^= effect;}}
        let rotated_state_arr = state_arr.mapv(|val| (val << 27) | (val >> (64 - 27)));
        for j in 0..16 {
            if j < rotated_state_arr.len() {
                let current_state_val = rotated_state_arr[j];
                let bytes_to_write = current_state_val.to_be_bytes();
                let start_index = j * 8;
                let end_index = (j + 1) * 8;
                if end_index <= hash_value.len() {
                    hash_value[start_index..end_index].copy_from_slice(&bytes_to_write);}}}}
    hash_value.truncate(64);
    hash_value}

pub fn generate_sbox_internal() -> SBoxType {
    let mut inverses = vec![0u16; 512];
    for i in 0..512 {
        if i == 0 {
            inverses[i] = 0;
        } else {
            let base_exp: u16 = (1 << 9) - 2;
            let key_val_u16 = (key_as_u128() >> (i % 113) as u32) as u16;
            let modifier = (i as u16)
                .wrapping_mul(key_val_u16)
                .rotate_left(((key_val_u16 % 9) as u32).max(1))
                .wrapping_add((MAGIC % 0x10000) as u16)
                .wrapping_sub((RATIO % 0x10000) as u16);
            let modified_exp = (base_exp ^ modifier) % ((1 << 9) - 1);
            let final_exp = if modified_exp == 0 { 1 } else { modified_exp };
            inverses[i] = gf2_9_pow(i as u16, final_exp, POLY_GF2_9_DEG9);}}
    let mut sbox = vec![0u16; 512];
    let key_bytes = QONST.to_be_bytes();
    let c_val_from_key = u16::from_be_bytes(key_bytes[0..2].try_into().unwrap_or([0, 0]))
        .wrapping_add(u16::from_be_bytes(key_bytes[8..10].try_into().unwrap_or([0, 0])))
        .rotate_left((key_bytes[15] % 9) as u32)
        .wrapping_mul(MAGIC as u16);
    let d_val_from_key = u16::from_be_bytes(key_bytes[2..4].try_into().unwrap_or([0, 0]))
        .wrapping_sub(u16::from_be_bytes(key_bytes[10..12].try_into().unwrap_or([0, 0])))
        .rotate_right((key_bytes[14] % 9) as u32)
        .wrapping_add(RATIO as u16);
    let c_affine_final = (c_val_from_key & 0x1FF) | 0x1;
    let d_affine_final = d_val_from_key & 0x1FF;
    for i in 0..512 {
        let affine_mult_part = gf2_9_mul(c_affine_final, inverses[i], POLY_GF2_9_DEG9);
        let mut t_intermediate = affine_mult_part ^ d_affine_final;
        t_intermediate = t_intermediate
            .wrapping_add(i as u16)
            .rotate_left((i % 9) as u32)
            .wrapping_mul(COEFFS[i % 5] as u16);
        t_intermediate ^= (key_as_u128() >> ((i / 16) % 8) * 8) as u16;
        let t_final = t_intermediate & 0x1FF;
        sbox[i] = t_final;}
    sbox}

pub fn gf2_9_mul(mut a: u16, mut b: u16, poly: u16) -> u16 {
    let mut res: u16 = 0;
    let nine_bit_mask: u16 = 0x1FF;
    a &= nine_bit_mask;
    b &= nine_bit_mask;
	debug!("gf2_9_mul: a={:x}, b={:x}, poly={:x}", a, b, poly);
    for _ in 0..9 {
        if (b & 1) != 0 {
            res ^= a;}
        let msb_a_set = (a & (1 << 8)) != 0;
        a <<= 1;
        if msb_a_set {
            a ^= poly;}
        b >>= 1;}
    debug!("gf2_9_mul: output={:x}", res);
    res & nine_bit_mask}

pub fn gf2_9_pow(mut base: u16, mut exp: u16, poly: u16) -> u16 {
    let mut res: u16 = 1;
    let nine_bit_mask: u16 = 0x1FF;
    base &= nine_bit_mask;
    debug!("gf2_9_pow: base={:x}, exp={:x}, poly={:x}", base, exp, poly);
    if base == 0 {
        return if exp == 0 { 1 } else { 0 };}
    while exp > 0 {
        if (exp & 1) != 0 {
            res = gf2_9_mul(res, base, poly);
            debug!("gf2_9_pow: exp is odd, res={:x}", res);}
        base = gf2_9_mul(base, base, poly);
        debug!("gf2_9_pow: base (squared)={:x}", base);
        exp >>= 1;}
    debug!("gf2_9_pow: output={:x}", res);
    res & nine_bit_mask}

#[target_feature(enable = "sse2")]
pub fn arx_internal(
    data_bytes: &[u8],
    constant_128: u128,
    magic_val: u64,
    ratio_val: u64,
    arx_bits: &[u32; 8],
) -> Vec<u64> {
    debug!("arx_internal: data_bytes_len={}, constant_128={:x}", data_bytes.len(), constant_128);
    let chunk_size = 8;
    let pad_len = (chunk_size - (data_bytes.len() % chunk_size)) % chunk_size;
    let mut padded_data = data_bytes.to_vec();
    padded_data.extend_from_slice(&vec![0; pad_len]);
    let mut output: Vec<u64> = Vec::with_capacity(padded_data.len() / chunk_size);
    let mut state_high = (constant_128 >> 64) as u64;
    let mut state_low = constant_128 as u64;
    for chunk in padded_data.chunks_exact(chunk_size) {
        let block = u64::from_be_bytes(chunk.try_into().unwrap());
        debug!("  arx_internal: processing chunk={:x}", block);
        state_low ^= block;
        for i in 0..12 {
            let rot1 = arx_bits[i % 8];
            let rot2 = arx_bits[(i + 1) % 8];
            state_low = state_low.wrapping_add(state_high);
            state_high = (state_high ^ state_low).rotate_left(rot1);
            state_low = state_low.wrapping_add(state_low.rotate_left(rot2));
            state_high ^= state_high.rotate_right(rot1);
            state_low = state_low.wrapping_mul(magic_val);
            state_high = state_high.wrapping_add(ratio_val.rotate_left((i as u32) * 5));
            state_high = state_high.wrapping_add(state_low);
            state_low = (state_low ^ state_high).rotate_left(rot2);
            debug!("  arx_internal: updated state={:x}", state_low);}
        output.push(state_low);}
    debug!("arx_internal: output_len={}, first_val={:x}", output.len(), output.get(0).unwrap_or(&0));
    output}

#[inline]
pub fn derive_internal(seed: u64, s_box: &SBoxType, internal_seed_param: &mut u128) -> u64 {
    let mut l: u64 = (seed >> 32) & MASK_32;
    let mut r: u64 = seed & MASK_32;
    for round_num in 0..32 {
        let rk_sbox_idx = ((r >> (32 - 9)) & 0x1FF) as usize;
        let sbox_output_for_rk = s_box[rk_sbox_idx % 512];
        let rk_term_sbox = (sbox_output_for_rk as u64).wrapping_mul(CONST);
        let rk_term_round_const = (round_num as u64).wrapping_mul(RATIO).rotate_left((round_num % 64) as u32);
        let rk = rk_term_sbox ^ rk_term_round_const ^ (r.rotate_left(round_num as u32 % 32));
        let rk_final = rk & MASK_32;
        let mut f_val = r.wrapping_add(rk_final).rotate_left((round_num as u32 * 3) % 32);
        f_val &= MASK_32;
        let f_sbox_idx = ((f_val >> 23) & 0x1FF) as usize;
        let sbox_output_for_f = s_box[f_sbox_idx % 512];
        f_val ^= (sbox_output_for_f as u64).wrapping_add(MAGIC);
        f_val &= MASK_32;
        let prev_l = l;
        l = r;
        r = prev_l ^ f_val;
        r = r.wrapping_add(CONST.rotate_right(round_num as u32 % 64));
        *internal_seed_param = internal_seed_param
            .wrapping_add(l as u128)
            .wrapping_mul(r as u128 | 1)
            .wrapping_add(key_as_u128() ^ (round_num as u128))
            .rotate_left((l % 128) as u32)
            .wrapping_sub((r as u128).rotate_right(round_num as u32 % 128));
        *internal_seed_param &= MASK_128;}
    let state_64 = (l << 32) | r;
    let term1 = state_64.wrapping_mul(MAGIC).wrapping_add(key_as_u64());
    let term2 = state_64.rotate_left(23) ^ RATIO.wrapping_add(CONST);
    let term3 = (state_64 >> 17).wrapping_add(key_as_u64()).rotate_right((CONST % 63) as u32);
    (term1 ^ term2).wrapping_add(term3).wrapping_mul(MAGIC ^ RATIO)}
