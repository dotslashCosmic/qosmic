// src/qosmic.rs
use hex;
use std::time::Instant;
use std::convert::TryInto;
use log::{debug, info};

use crate::constants::*;
use crate::primitives::{
    generate_sbox_internal, arx_internal, derive_internal
};
use crate::components::{
    v_func_internal, w_func_internal, d_func_internal, h_func_internal
};

pub type SBoxType = Vec<u16>;

lazy_static! {
    pub static ref SBOX: SBoxType = {
        info!("Generating S-Box...");
        let start = Instant::now();
        let sbox = generate_sbox_internal();
        info!("S-Box generation took: {:?}", start.elapsed());
        sbox
    };
}

pub fn get_sbox() -> &'static SBoxType {
    &SBOX
}

fn arx(data_bytes: &[u8]) -> Vec<u64> {
    unsafe {
        arx_internal(data_bytes, KEY, MAGIC, RATIO, &ARX_BITS)
    }
}

pub(crate) fn v_func(x_input: u64, nonce: u64, p_array_val: &[u64; 5]) -> u64 {
    unsafe {
        v_func_internal(x_input, nonce, p_array_val)
    }
}

pub(crate) fn w_func(x: u64, y: u64, internal_seed: &mut u128) -> u64 {
    w_func_internal(x, y, internal_seed)
}

pub(crate) fn d_func(x:u64, y:u64, z:u64, internal_seed: &mut u128, main_state_arr: &[u64; 8]) -> u64 {
    d_func_internal(x,y,z, internal_seed, main_state_arr)
}

fn h_func(
    a_in: u64,
    b_in: u64,
    c_in: u64,
    d_in: u64,
    seed_h: u64,
    internal_seed: &mut u128,
    nonce: u64,
    p_array: &[u64;5],
    main_state_arr: &[u64; 8]
) -> (u64, u64, u64, u64) {
    h_func_internal(a_in, b_in, c_in, d_in, seed_h, internal_seed, nonce, p_array, main_state_arr)
}


pub fn qosmic512(
    mut input_data_bytes: Vec<u8>,
    _fs_char: char,
    _s_box_param: &SBoxType,
    _small_primes: &[u64],
    nonce: u64,
) -> String {
    let total_hash_start_time = Instant::now();
    let split_len: usize = 64;
    let mut current_main_state: [u64; 8] = [0; 8];
    let mut internal_seed: u128 = KEY & MASK_128;
    debug!("qosmic_512 internal_seed (initial): {:x}", internal_seed);
    debug!("Nonce for this run: {}", nonce);

    let _init_start_time = Instant::now();

    let p_array: [u64; 5] = {
        let mut arr = [0u64; 5];
        for i in 0..5 {
            arr[i] = nonce.wrapping_add(COEFFS[i]).wrapping_mul(RATIO).wrapping_add(i as u64) | 1;
        }
        arr
    };

    let padding_start_time = Instant::now();
    let data_len = input_data_bytes.len();
    let initial_pad_len = (split_len - (data_len % split_len)) % split_len;
    let mut final_pad_len = initial_pad_len;
    if final_pad_len < 9 {
        final_pad_len += split_len;
    }

    input_data_bytes.push(0x80);

    let num_zero_bytes = final_pad_len.saturating_sub(1 + 8);
    for _ in 0..num_zero_bytes {
        input_data_bytes.push(0u8);
    }

    input_data_bytes.extend_from_slice(&(data_len as u64).to_be_bytes());
    debug!("Appended original length ({} bytes). Final padded data length: {}", data_len, input_data_bytes.len());
	debug!("Padding took: {:?}", padding_start_time.elapsed());

    let chunk_processing_start_time = Instant::now();
    let mut _chunk_idx = 0;
    for chunk_bytes in input_data_bytes.chunks_exact(split_len) {
        let arx_output_u64s: Vec<u64> = arx(chunk_bytes);

        for i in 0..current_main_state.len() {
            if i < arx_output_u64s.len() {
                current_main_state[i] ^= arx_output_u64s[i];
            }
        }

        let chunk_1_u64_bytes: [u8; 8] = chunk_bytes[56..64].try_into().unwrap();
        let chunk_1_u64 = u64::from_be_bytes(chunk_1_u64_bytes);

        let chunk_2_u64_bytes: [u8; 8] = chunk_bytes[0..8].try_into().unwrap();
        let chunk_2_u64 = u64::from_be_bytes(chunk_2_u64_bytes) ^ MAGIC;


        let (a, b, c, d) = h_func(
            current_main_state[0], current_main_state[1], current_main_state[2], current_main_state[3],
            chunk_1_u64,
            &mut internal_seed, nonce, &p_array, &current_main_state
        );


        let (e, f, g, h_val) = h_func(
            current_main_state[4], current_main_state[5], current_main_state[6], current_main_state[7],
            chunk_2_u64,
            &mut internal_seed, nonce, &p_array, &current_main_state
        );

        current_main_state[0] = a; current_main_state[1] = b;
        current_main_state[2] = c; current_main_state[3] = d;
        current_main_state[4] = e; current_main_state[5] = f;
        current_main_state[6] = g; current_main_state[7] = h_val;
        _chunk_idx += 1;
    }
    debug!("Total chunk processing took: {:?}", chunk_processing_start_time.elapsed());

    let finalization_start_time = Instant::now();

    let state_bytes_conversion_start = Instant::now();
    let final_hash_state_bytes: Vec<u8> = current_main_state.iter()
                                                     .flat_map(|&x| x.to_be_bytes())
                                                     .collect();
    let state_bytes_conversion_duration = state_bytes_conversion_start.elapsed();
    debug!("  State bytes conversion took: {:?}", state_bytes_conversion_duration);
    debug!("Final hash state bytes (first 16): {:?}", &final_hash_state_bytes[..16]);

    let salt_gen_start = Instant::now();
    let salt_seed = nonce ^ current_main_state[0] ^ current_main_state[7];
    debug!("Salt seed for final derive: {}", salt_seed);
    let salt_derived_val = derive_internal(salt_seed, &SBOX);

    debug!("Salt derived value: {}", salt_derived_val);

    let mut salt_bytes_padded = vec![0u8; 64];
    let salt_bytes = salt_derived_val.to_be_bytes();

    let start_copy_idx = 64 - salt_bytes.len();
    salt_bytes_padded[start_copy_idx..].copy_from_slice(&salt_bytes);
    let salt_gen_duration = salt_gen_start.elapsed();
    debug!("Salt generation and padding took: {:?}", salt_gen_duration);
    debug!("Padded salt bytes (first 16): {:?}", &salt_bytes_padded[..16]);
    
    let sbox_combine_start = Instant::now();
    let mut final_qosmic_bytes: Vec<u8> = Vec::with_capacity(64);

    for i in 0..64 {
        let hash_byte = final_hash_state_bytes[i];
        let salt_byte = salt_bytes_padded[i];

        let combined_byte = hash_byte ^ salt_byte;

        let sbox_index = combined_byte as usize % 512;
        let sbox_output_u16 = SBOX[sbox_index];
        let transformed_byte = (sbox_output_u16 & 0xFF) as u8;

        final_qosmic_bytes.push(transformed_byte);
    }
    let sbox_combine_duration = sbox_combine_start.elapsed();
    debug!("S-Box combination and transformation took: {:?}", sbox_combine_duration);
    debug!("Final qosmic bytes (first 16): {:?}", &final_qosmic_bytes[..16]);
    
    let hex_result = hex::encode(final_qosmic_bytes);
    
    let finalization_total_duration = finalization_start_time.elapsed();

    let total_sub_timers_duration = state_bytes_conversion_duration
                                         .checked_add(salt_gen_duration).unwrap_or_default()
                                         .checked_add(sbox_combine_duration).unwrap_or_default();

    let estimated_overhead = finalization_total_duration
                                 .checked_sub(total_sub_timers_duration)
                                 .unwrap_or_default();

    debug!("Estimated untimed overhead: {:?}", estimated_overhead);
    debug!("Finalization total took: {:?}", finalization_total_duration);

    let total_hash_duration = total_hash_start_time.elapsed();
    info!("--- qosmic_512 hash time: {:?} ---", total_hash_duration);
    hex_result
}
