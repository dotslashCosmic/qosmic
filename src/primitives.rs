// src/primitives.rs
use ndarray::{Array1, Array2, Axis};
use rand::Rng;
use std::convert::TryInto;
use crate::constants::*;
use crate::qosmic::SBoxType;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;


const POLY_GF2_9_DEG9: u16 = 0x211;

const C_AFFINE_9BIT: u16 = 0x1A5;
const D_AFFINE_9BIT: u16 = 0x069;


pub fn encrypt_internal(
    message_bits: &Array1<u8>,
) -> (Array2<u64>, Array1<u64>) {
    let num_bits = message_bits.len();
    let mut rng = rand::thread_rng();

    let mut a_flat = Vec::with_capacity(DIM * num_bits);
    for _ in 0..(DIM * num_bits) {
        a_flat.push(rng.gen_range(0..Q_MOD));
    }
    let a_list = Array2::from_shape_vec((num_bits, DIM), a_flat)
        .expect("Failed to reshape a_list for LWE matrix A");

    let mut errors_vec = Vec::with_capacity(num_bits);
    for _ in 0..num_bits {
        errors_vec.push(rng.gen_range(-MAGNITUDE..(MAGNITUDE + 1)));
    }
    let errors = Array1::from_vec(errors_vec);
    
    let a_sq = a_list.mapv(|x| (x as u128 * x as u128 % Q_MOD as u128) as u64);
    let sum_a_sq: Array1<u64> = a_sq.sum_axis(Axis(1));
    let message_bits_u64 = message_bits.mapv(|x| x as u64);
    let term3_message_encoding = message_bits_u64.mapv(|bit| (Q_MOD / 2) * bit);

    let b_ciphertext = sum_a_sq.iter().zip(errors.iter()).zip(term3_message_encoding.iter())
        .map(|((&sa_sq_val, &err_val), &term3_val)| {
            let sa_sq_mod_q = sa_sq_val % Q_MOD;
            let term1_plus_2_mod_q = (sa_sq_mod_q as i128 + err_val as i128 + Q_MOD as i128) % Q_MOD as i128;
            let result = (term1_plus_2_mod_q as u128 + term3_val as u128) % Q_MOD as u128;
            result as u64
        })
        .collect::<Array1<u64>>();

    (a_list, b_ciphertext)
}

pub fn quantum_internal(input_data: &[u8]) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    
    let mut hash_value = vec![0u8; 128];
    let input_len = input_data.len();
    let block_size: usize = 4096;

    for i in (0..input_len).step_by(block_size) {
        let end = (i + block_size).min(input_len);
        let mut block_bytes = input_data[i..end].to_vec();
        if block_bytes.len() < block_size {
            block_bytes.resize(block_size, 0x00);
        }

        let mut bits_unpacked = Vec::new();
        for byte_val in &block_bytes {
            for k in 0..8 {
                bits_unpacked.push((byte_val >> (7 - k)) & 1);
            }
        }
        bits_unpacked.truncate(DIM);
        let bits_arr = Array1::from_vec(bits_unpacked);

        let (_a_list, b_values) = encrypt_internal(&bits_arr);

        let mut state_u64 = Vec::with_capacity(block_size / 8);
        for chunk in block_bytes.chunks_exact(8) {
            state_u64.push(u64::from_be_bytes(chunk.try_into().unwrap()));
        }
        let mut state_arr = Array1::from_vec(state_u64);

        let state_arr_len = state_arr.len();
        if state_arr_len > 0 {
            for (idx, &b_val) in b_values.iter().enumerate() {
                let effect = b_val % Q_MOD;
                state_arr[idx % state_arr_len] ^= effect;
            }
        }
        
        let rotated_state_arr = state_arr.mapv(|val| (val << 27) | (val >> (64-27)));

        for j in 0..16 {
            if j < rotated_state_arr.len() {
                let current_state_val = rotated_state_arr[j];
                let bytes_to_write = current_state_val.to_be_bytes();
                let start_index = j * 8;
                let end_index = (j + 1) * 8;
                if end_index <= hash_value.len() {
                    hash_value[start_index..end_index].copy_from_slice(&bytes_to_write);
                }
            }
        }
    }
    hash_value.truncate(64);
    hash_value
}

pub fn generate_sbox_internal() -> SBoxType {
    let mut inverses = vec![0u16; 512];
    for i in 0..512 {
        if i == 0 {
            inverses[i] = 0;
        } else {
            inverses[i] = gf2_9_pow(i as u16, (1 << 9) - 2, POLY_GF2_9_DEG9);
        }
    }

    let mut sbox = vec![0u16; 512];
    let key_bytes = KEY.to_be_bytes();
    let key_len = key_bytes.len();

    for i in 0..512 {
        let key_byte_for_xor = key_bytes[i % key_len] as u16;
        let t_intermediate = inverses[i] ^ key_byte_for_xor;
        
        let affine_mult_part = gf2_9_mul(C_AFFINE_9BIT, t_intermediate, POLY_GF2_9_DEG9);
        let t_final = (affine_mult_part ^ D_AFFINE_9BIT) & 0x1FF;
        
        sbox[i] = t_final;
    }

    for i in 0..512 {
        if sbox[i] == (i as u16) {
            let swap_idx = (i + 1) % 512;
            sbox.swap(i, swap_idx);
        }
    }
    sbox
}

pub fn gf2_9_mul(mut a: u16, mut b: u16, poly: u16) -> u16 {
    let mut res: u16 = 0;
    let nine_bit_mask: u16 = 0x1FF;
    a &= nine_bit_mask;
    b &= nine_bit_mask;

    for _ in 0..9 {
        if (b & 1) != 0 {
            res ^= a;
        }
        let msb_a_set = (a & (1 << 8)) != 0;
        a <<= 1;
        if msb_a_set {
            a ^= poly;
        }
        b >>= 1;
    }
    res & nine_bit_mask
}

pub fn gf2_9_pow(mut base: u16, mut exp: u16, poly: u16) -> u16 {
    let mut res: u16 = 1;
    let nine_bit_mask: u16 = 0x1FF;
    base &= nine_bit_mask;

    if base == 0 { return if exp == 0 { 1 } else { 0 }; }

    while exp > 0 {
        if (exp & 1) != 0 {
            res = gf2_9_mul(res, base, poly);
        }
        base = gf2_9_mul(base, base, poly);
        exp >>= 1;
    }
    res & nine_bit_mask
}

#[target_feature(enable = "sse2")]
pub fn arx_internal(data_bytes: &[u8], key_val: u128, magic_val: u64, ratio_val: u64, arx_bits_val: &[u32]) -> Vec<u64> {
    let pad_len = (8 - (data_bytes.len() % 8)) % 8;
    let mut padded_bytes = data_bytes.to_vec();
    
    for _ in 0..pad_len {
        padded_bytes.push(pad_len as u8);
    }
    
    let mut blocks: Vec<u64> = Vec::with_capacity(padded_bytes.len() / 8);

    #[cfg(target_arch = "x86_64")]
    unsafe {
        for chunk in padded_bytes.chunks_exact(16) {
            let block_pair = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
            blocks.push(_mm_cvtsi128_si64(block_pair) as u64);
            blocks.push(_mm_cvtsi128_si64(_mm_srli_si128(block_pair, 8)) as u64);
        }
    }
    for chunk in padded_bytes[blocks.len() * 8 ..].chunks_exact(8) {
        blocks.push(u64::from_be_bytes(chunk.try_into().unwrap()));
    }


    let rounds = arx_bits_val.len();
    let round_keys: Vec<u64> = (0..rounds)
        .map(|r| {
            let key_low_64 = (key_val & MASK_64 as u128) as u64;
            let key_high_64 = ((key_val >> 64) & MASK_64 as u128) as u64;
            
            let term1 = key_low_64.wrapping_add(magic_val.wrapping_shl((r % 8) as u32 * 7));
            let term2 = key_high_64 ^ ratio_val.rotate_right((r % 8) as u32 * 5);
            (term1 ^ term2).wrapping_add((r as u64).wrapping_mul(RATIO))
        })
        .collect();

    for (i, &rot_amount) in arx_bits_val.iter().enumerate() {
        let round_key_i = round_keys[i];
        let effective_rot = rot_amount % 64;
        
        for block_val in blocks.iter_mut() {
            let rotated_val = block_val.rotate_left(effective_rot);
            *block_val = block_val.wrapping_add(rotated_val) ^ round_key_i;
        }
        
        let key_high_64_transformed = ((key_val >> 64) & MASK_64 as u128) as u64;
        let inner_round_key = key_high_64_transformed.rotate_left((i % 8 * 3) as u32);
        
        for block_val in blocks.iter_mut() {
            let mut current_block_processing = *block_val;
            current_block_processing ^= inner_round_key;
            current_block_processing = current_block_processing.rotate_left((effective_rot + 3) % 64);
            *block_val = current_block_processing;
        }
    }
    blocks
}

#[inline]
pub fn derive_internal(seed: u64, s_box: &SBoxType) -> u64 {
    let mut l: u64 = (seed >> 32) & MASK_32;
    let mut r: u64 = seed & MASK_32;

    for round_num in 0..16 {
        let rk_sbox_idx = ((r >> (32 - 9)) & 0x1FF) as usize;
        let sbox_output_for_rk = s_box[rk_sbox_idx % 512];

        let rk_byte_from_sbox = (sbox_output_for_rk & 0xFF) as u64;
        let rk_byte1_from_r = (r >> 24) & 0xFF;
        let rk_byte2_from_r = (r >> 16) & 0xFF;
        let rk_byte3_from_r = (r >> 8) & 0xFF;
        
        let rk_term_round_const = (round_num as u64).wrapping_mul(RATIO);
        let rk = (rk_byte_from_sbox << 24) ^
                     (rk_byte1_from_r << 16) ^
                     (rk_byte2_from_r << 8) ^
                     (rk_byte3_from_r) ^
                     (rk_term_round_const & MASK_32);
        let rk_final = rk & MASK_32;


        let mut f_val = r.wrapping_add(rk_final);
        f_val &= MASK_32;
        
        f_val = f_val.rotate_left(7);
        f_val &= MASK_32;

        let f_sbox_idx = ((f_val >> (32 - 9)) & 0x1FF) as usize;
        let sbox_output_for_f = s_box[f_sbox_idx % 512];
        
        f_val ^= (sbox_output_for_f & 0xFF) as u64;
        f_val &= MASK_32;

        let prev_l = l;
        l = r;
        r = prev_l ^ f_val;
    }
    let state_64 = (l << 32) | r;
    
    let term1 = state_64.wrapping_mul(MAGIC);
    let term2 = state_64.rotate_left(23) ^ RATIO;
    let term3 = (state_64 >> 17).wrapping_add(KEY as u64);
    
    (term1 ^ term2).wrapping_add(term3)
}
