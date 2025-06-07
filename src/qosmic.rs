// src/qosmic.rs
use hex;
use std::time::Instant;
use std::convert::TryInto;
use log::{debug, info};

use crate::constants::*;
use crate::primitives::{
    generate_sbox_internal, arx_internal, derive_internal};
use crate::components::{
    v_func_internal, w_func_internal, d_func_internal, h_func_internal,
    permute_1_internal, permute_2_internal};

pub type SBoxType = Vec<u16>;

lazy_static! {
    pub static ref SBOX: SBoxType = {
        info!("Generating S-Box...");
        let start = Instant::now();
        let sbox = generate_sbox_internal();
        info!("S-Box generation took: {:?}", start.elapsed());
        sbox};}

pub fn get_sbox() -> &'static SBoxType {
    &SBOX}

fn arx(data_bytes: &[u8]) -> Vec<u64> {
    unsafe {
        arx_internal(data_bytes, KEY, MAGIC, RATIO, &ARX_BITS)}}

pub(crate) fn v_func(x_input: u64, nonce: u64, p_array_val: &[u64; 5]) -> u64 {
    v_func_internal(x_input, nonce, p_array_val)}

pub(crate) fn w_func(x: u64, y: u64, internal_seed: &mut u128) -> u64 {
    w_func_internal(x, y, internal_seed)}

pub(crate) fn d_func(x:u64, y:u64, z:u64, internal_seed: &mut u128, main_state_arr: &[u64; 8]) -> u64 {
    d_func_internal(x,y,z, internal_seed, main_state_arr)}

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
    h_func_internal(a_in, b_in, c_in, d_in, seed_h, internal_seed, nonce, p_array, main_state_arr)}

#[inline]
fn final_byte_transform(combined_byte: u8, sbox_output_u16: u16, internal_seed: &mut u128) -> u8 {
    let mut transformed_val = sbox_output_u16 as u8;
    transformed_val = transformed_val.wrapping_add((sbox_output_u16 >> 8) as u8);
    transformed_val ^= combined_byte;
    let seed_low_byte = (*internal_seed & 0xFF) as u8;
    let seed_high_byte = ((*internal_seed >> 8) & 0xFF) as u8;
    transformed_val = transformed_val.rotate_left((seed_low_byte % 8) as u32);
    transformed_val ^= seed_high_byte.wrapping_add(combined_byte.rotate_right(3));
    transformed_val = transformed_val.wrapping_mul(MAGIC as u8).wrapping_add(RATIO as u8);
    *internal_seed = internal_seed.wrapping_add(transformed_val as u128)
                                .wrapping_mul(combined_byte as u128 | 1)
                                .wrapping_add(KEY ^ (MAGIC as u128));
    *internal_seed &= MASK_128;
    transformed_val}

pub fn qosmic512(
    mut input_data_bytes: Vec<u8>,
    _fs_char: char, 
    _s_box_param: &SBoxType, 
    _small_primes: &[u64], 
    nonce: u64,
    key: Option<&[u8]>,
) -> String {
    let total_hash_start_time = Instant::now();
    let split_len: usize = 64; 
    let mut current_main_state: [u64; 8] = [0; 8]; 
    let mut internal_seed: u128 = KEY & MASK_128; 
    if let Some(key_bytes) = key {
        debug!("Mixing in user-provided key of length {} bytes.", key_bytes.len());
        let mut key_seed_mixer: u128 = 0;
        for chunk in key_bytes.chunks(16) {
            let mut temp_chunk: [u8; 16] = [0; 16];
            temp_chunk[..chunk.len()].copy_from_slice(chunk);
            let chunk_val = u128::from_be_bytes(temp_chunk);
            key_seed_mixer = key_seed_mixer.wrapping_add(chunk_val);
            key_seed_mixer ^= key_seed_mixer.rotate_left(37)
                               .wrapping_mul(RATIO as u128)
                               .wrapping_add(MAGIC as u128);}
        internal_seed ^= key_seed_mixer;}
    debug!("qosmic_512 internal_seed (initial): {:x}", internal_seed);
    debug!("Nonce for this run: {}", nonce);
    let _init_start_time = Instant::now(); 
    let transformed_nonce = nonce.wrapping_mul(MAGIC).rotate_left((nonce % 64) as u32).wrapping_add(RATIO);
    let p_array: [u64; 5] = {
        let mut arr = [0u64; 5];
        for i in 0..5 {
            arr[i] = transformed_nonce.wrapping_add(COEFFS[i]).wrapping_mul(RATIO).wrapping_add(i as u64) | 1;}
        arr};
    let padding_start_time = Instant::now();
    let data_len = input_data_bytes.len();
    let initial_pad_len = (split_len - (data_len % split_len)) % split_len;
    let mut final_pad_len = initial_pad_len;
    if final_pad_len < 9 {
        final_pad_len += split_len;}
    input_data_bytes.push(0x80); 
    let num_zero_bytes = final_pad_len.saturating_sub(1 + 8); 
    for _ in 0..num_zero_bytes {
        input_data_bytes.push(0u8);}
    input_data_bytes.extend_from_slice(&(data_len as u64).to_be_bytes()); 
    debug!("Appended original length ({} bytes). Final padded data length: {}", data_len, input_data_bytes.len());
	debug!("Padding took: {:?}", padding_start_time.elapsed());
    let chunk_processing_start_time = Instant::now();
    let mut _chunk_idx = 0; 
    for chunk_bytes in input_data_bytes.chunks_exact(split_len) {
        let arx_output_u64s: Vec<u64> = arx(chunk_bytes);
        for i in 0..current_main_state.len() {
            if i < arx_output_u64s.len() {
                current_main_state[i] ^= arx_output_u64s[i];}}
        let chunk_1_u64_bytes: [u8; 8] = chunk_bytes[56..64].try_into().unwrap();
        let chunk_1_u64 = u64::from_be_bytes(chunk_1_u64_bytes);
        let chunk_2_u64_bytes: [u8; 8] = chunk_bytes[0..8].try_into().unwrap();
        let chunk_2_u64 = u64::from_be_bytes(chunk_2_u64_bytes) ^ MAGIC; 
        let (a, b, c, d) = h_func(
            current_main_state[0], current_main_state[1], current_main_state[2], current_main_state[3],
            chunk_1_u64, 
            &mut internal_seed, nonce, &p_array, &current_main_state);
        let (e, f, g, h_val) = h_func(
            current_main_state[4], current_main_state[5], current_main_state[6], current_main_state[7],
            chunk_2_u64, 
            &mut internal_seed, nonce, &p_array, &current_main_state);
        current_main_state[0] = a; current_main_state[1] = b;
        current_main_state[2] = c; current_main_state[3] = d;
        current_main_state[4] = e; current_main_state[5] = f;
        current_main_state[6] = g; current_main_state[7] = h_val;
        internal_seed = internal_seed.wrapping_add(arx_output_u64s[0] as u128)
                                     .wrapping_mul(current_main_state[0] as u128 | 1)
                                     .rotate_left((current_main_state[1] % 128) as u32)
                                     .wrapping_add(KEY ^ (RATIO as u128))
                                     .wrapping_sub(current_main_state[7] as u128 ^ (MAGIC as u128).rotate_right(11));
        internal_seed &= MASK_128;
        let (x_m, y_m, z_m, w_m) = h_func(
            current_main_state[0] ^ current_main_state[4].rotate_left(1),
            current_main_state[1] ^ current_main_state[5].rotate_left(3),
            current_main_state[2] ^ current_main_state[6].rotate_left(5),
            current_main_state[3] ^ current_main_state[7].rotate_left(7),
            nonce.wrapping_add(_chunk_idx as u64).wrapping_add(arx_output_u64s[arx_output_u64s.len() / 2]),
            &mut internal_seed, nonce.wrapping_add(_chunk_idx as u64).rotate_left(5).wrapping_add(arx_output_u64s[arx_output_u64s.len() / 4]),
            &p_array, &current_main_state);
        current_main_state[0] ^= x_m.wrapping_add(MAGIC); current_main_state[1] ^= y_m.wrapping_sub(RATIO);
        current_main_state[2] ^= z_m.rotate_left(13); current_main_state[3] ^= w_m.rotate_right(17);
        current_main_state[4] ^= x_m.rotate_left(17); current_main_state[5] ^= y_m.rotate_right(19);
        current_main_state[6] ^= z_m.wrapping_add(RATIO); current_main_state[7] ^= w_m.wrapping_sub(MAGIC);
        _chunk_idx += 1;}
    debug!("Total chunk processing took: {:?}", chunk_processing_start_time.elapsed());
    let finalization_start_time = Instant::now();
    let (final_a, final_b, final_c, final_d) = h_func(
        current_main_state[0] ^ current_main_state[4],
        current_main_state[1] ^ current_main_state[5],
        current_main_state[2] ^ current_main_state[6],
        current_main_state[3] ^ current_main_state[7],
        nonce.wrapping_add(MAGIC), 
        &mut internal_seed, nonce.wrapping_add(RATIO).rotate_left(7), &p_array, &current_main_state);
    current_main_state[0] = final_a; current_main_state[1] = final_b;
    current_main_state[2] = final_c; current_main_state[3] = final_d;
    current_main_state[4] ^= final_a.rotate_left(31);
    current_main_state[5] ^= final_b.rotate_right(27);
    current_main_state[6] ^= final_c.wrapping_add(MAGIC);
    current_main_state[7] ^= final_d.wrapping_sub(RATIO);
    let state_bytes_conversion_start = Instant::now();
    let mut final_hash_state_bytes: Vec<u8> = current_main_state.iter()
                                                     .flat_map(|&x| x.to_be_bytes())
                                                     .collect();
    let state_bytes_conversion_duration = state_bytes_conversion_start.elapsed();
    debug!("  State bytes conversion took: {:?}", state_bytes_conversion_duration);
    debug!("Final hash state bytes (first 16): {:?}", &final_hash_state_bytes[..16]);
    for _ in 0..8 {
        permute_1_internal(&mut final_hash_state_bytes, &mut internal_seed);
        permute_2_internal(&mut final_hash_state_bytes, &mut internal_seed);}
    debug!("  After final permutations (first 16): {:?}", &final_hash_state_bytes[..16]);
    let mut temp_state_u64: [u64; 8] = [0; 8];
    for i in 0..8 {
        temp_state_u64[i] = u64::from_be_bytes(final_hash_state_bytes[i*8..(i+1)*8].try_into().unwrap());}
    let (compressed_a, compressed_b, compressed_c, compressed_d) = h_func(
        temp_state_u64[0] ^ temp_state_u64[4],
        temp_state_u64[1] ^ temp_state_u64[5],
        temp_state_u64[2] ^ temp_state_u64[6],
        temp_state_u64[3] ^ temp_state_u64[7],
        nonce.wrapping_add(CONST),
        &mut internal_seed, nonce.wrapping_add(KEY as u64).rotate_right(13),
        &p_array, &temp_state_u64);
    let mut final_compressed_bytes: Vec<u8> = Vec::with_capacity(64);
    final_compressed_bytes.extend_from_slice(&compressed_a.to_be_bytes());
    final_compressed_bytes.extend_from_slice(&compressed_b.to_be_bytes());
    final_compressed_bytes.extend_from_slice(&compressed_c.to_be_bytes());
    final_compressed_bytes.extend_from_slice(&compressed_d.to_be_bytes());
    final_compressed_bytes.extend_from_slice(&temp_state_u64[4].wrapping_add(compressed_a).to_be_bytes());
    final_compressed_bytes.extend_from_slice(&temp_state_u64[5].wrapping_add(compressed_b).to_be_bytes());
    final_compressed_bytes.extend_from_slice(&temp_state_u64[6].wrapping_add(compressed_c).to_be_bytes());
    final_compressed_bytes.extend_from_slice(&temp_state_u64[7].wrapping_add(compressed_d).to_be_bytes());
    final_hash_state_bytes = final_compressed_bytes;
    debug!("  After post-permutation h_func compression (first 16): {:?}", &final_hash_state_bytes[..16]);
    let salt_gen_start = Instant::now();
    let salt_seed = nonce ^ current_main_state[0] ^ current_main_state[7]; 
    debug!("Salt seed for final derive: {}", salt_seed);
    let salt_derived_val = derive_internal(salt_seed, &SBOX, &mut internal_seed); 
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
        let transformed_byte = final_byte_transform(combined_byte, sbox_output_u16, &mut internal_seed);
        final_qosmic_bytes.push(transformed_byte);}
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
    hex_result}
