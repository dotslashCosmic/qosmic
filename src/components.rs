// src/components.rs
use crate::constants::*;
use crate::utils;
use crate::qosmic;
use log::debug;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub type BitGenerator = Box<dyn FnMut() -> u8>;

#[inline]
pub fn v_func_internal(mut x: u64, nonce: u64, p_array_val: &[u64; 5]) -> u64 {
    for i in 0..32 {
        x = x.wrapping_add(nonce);
        let rot_amount_1 = 17u32.wrapping_add((nonce as u32).wrapping_mul(i as u32) % 64);
        x = x.wrapping_add(x.rotate_left(rot_amount_1));
        let rot_amount_2 = 11u32.wrapping_add((i as u32).wrapping_mul((MAGIC % 64) as u32));
        x ^= x.rotate_left(rot_amount_2);
        let rot_amount_3 = 7u32.wrapping_add((i as u32).wrapping_mul((RATIO % 64) as u32));
        x ^= x.rotate_left(rot_amount_3);
        x = x.wrapping_add(MAGIC);
        let rot_amount_4 = 5u32.wrapping_add((i as u32).wrapping_mul(KEY as u32 % 64));
        x ^= x.rotate_left(rot_amount_4);
        x = x.wrapping_add(RATIO ^ (KEY as u64));}
    let exponent_val = COEFFS[0] | 0x10001;
    let modulus_val_p0 = p_array_val[0];
    if modulus_val_p0 < 2 {
        debug!("Warning: v_func_internal received modulus_val_p0 < 2. Using fallback Q_MOD for pow_mod.");
        return utils::pow_mod(x, exponent_val, Q_MOD); }
    utils::pow_mod(x, exponent_val, modulus_val_p0)}

#[inline]
pub fn w_func_internal(a: u64, b: u64, internal_seed: &mut u128) -> u64 {
    let mut result = a.wrapping_add(b).wrapping_mul(RATIO);
    let seed_low = (*internal_seed & MASK_64 as u128) as u64;
    let seed_high = ((*internal_seed >> 64) & MASK_64 as u128) as u64;
    result = result.wrapping_add(seed_low).rotate_left(13);
    result ^= seed_high.wrapping_sub(MAGIC);
    result = result.rotate_left(7).wrapping_add(b);
    result ^= result >> 13;
    result = result.wrapping_mul(MAGIC).wrapping_add(CONST);
    result ^= (a.wrapping_sub(b)).rotate_right(19).wrapping_sub(KEY as u64);
    result = result.wrapping_add(RATIO).rotate_left(5);
    result ^= (result >> 17).wrapping_mul(CONST);
    *internal_seed = internal_seed.wrapping_add(result as u128)
                                .wrapping_mul(b as u128 | 1) 
                                .wrapping_add(KEY ^ ((*internal_seed >> 3) as u128) ^ (MAGIC as u128));
    *internal_seed &= MASK_128;
    result}

#[inline]
pub fn d_func_internal(x: u64, y: u64, z: u64, internal_seed: &mut u128, main_state_arr: &[u64; 8]) -> u64 {
    let mut output = x.wrapping_add(y).wrapping_add(z).wrapping_mul(RATIO);
    output ^= (*internal_seed & MASK_64 as u128) as u64;
    output = output.rotate_right(5).wrapping_add(CONST);
    for k in 0..4 {
        let state_val_k = main_state_arr[k % 8];
        output ^= state_val_k;
        output = output.wrapping_mul(RATIO.wrapping_add(state_val_k));
        output = output.rotate_left(state_val_k.wrapping_rem(63) as u32 + 1);
        output ^= ((*internal_seed >> (k * 16 % 128)) & MASK_64 as u128) as u64;
        output = output.wrapping_add(MAGIC.rotate_right((k % 64) as u32));}
    output = output.wrapping_sub(x.wrapping_add(y).wrapping_mul(z)).rotate_right(11);
    output ^= ((*internal_seed >> 32) as u64).wrapping_add(CONST);
    *internal_seed = internal_seed.wrapping_add(output as u128)
                                .wrapping_mul(main_state_arr[0] as u128 | 1) 
                                .wrapping_add(main_state_arr[7] as u128)
                                .wrapping_sub(KEY ^ (RATIO as u128));
    *internal_seed &= MASK_128;
    output}

#[inline]
pub fn h_func_internal(
    mut a: u64, 
    mut b: u64, 
    mut c: u64, 
    mut d: u64, 
    seed_h: u64,
    internal_seed: &mut u128,
    nonce: u64, 
    p_array: &[u64;5],
    main_state_arr: &[u64; 8]
) -> (u64, u64, u64, u64) {
    a = a.wrapping_add(seed_h).wrapping_mul(RATIO);
    b = b.wrapping_sub(nonce).rotate_left(ARX_BITS[1]);
    c = c.wrapping_add(CONST).rotate_right(ARX_BITS[2]);
    d = d.wrapping_sub(MAGIC).wrapping_add(nonce);
    for i in 0..4 {
        a = qosmic::v_func(a, nonce.wrapping_add(i as u64), p_array);
        b = qosmic::w_func(b, a.wrapping_add(i as u64), internal_seed);
        c = qosmic::d_func(c, b.wrapping_add(i as u64), a.wrapping_add(i as u64), internal_seed, main_state_arr);
        d = d.wrapping_add(c).rotate_left(ARX_BITS[i % 8].wrapping_add(nonce as u32));
        (a, b, c, d) = (
            b ^ c.wrapping_add(d).rotate_left(ARX_BITS[(i+1) % 8]),
            d.wrapping_add(a).wrapping_mul(RATIO),
            a.rotate_left(13).wrapping_add(b).wrapping_sub(MAGIC),
            b.wrapping_mul(KEY as u64 ^ (nonce.rotate_right(i as u32 % 64))) ^ c.rotate_left(ARX_BITS[(i+2)%8]));}
    let seed_part_a = ((*internal_seed).wrapping_add(KEY) & MASK_64 as u128) as u64;
    let seed_part_b = (((*internal_seed).wrapping_mul(RATIO as u128)) >> 64 & MASK_64 as u128) as u64;
    a ^= seed_part_a.rotate_left(7);
    b = b.wrapping_add(seed_part_b.rotate_left(3)).wrapping_mul(CONST);
    c = c.wrapping_sub(seed_part_a.rotate_right(11)).wrapping_add(MAGIC);
    d = d.wrapping_mul(seed_part_b.rotate_right(19)).wrapping_sub(RATIO);
    a = a.rotate_left(13).wrapping_add(b);
    b = b.rotate_right(17).wrapping_add(c);
    c = c.wrapping_add(d).rotate_left(19);
    d = d.wrapping_add(a).rotate_right(23);
    *internal_seed = internal_seed.wrapping_add(a as u128)
                                .wrapping_mul(b as u128)
                                .wrapping_add(c as u128)
                                .wrapping_sub(d as u128)
                                .wrapping_add(CONST as u128)
                                .wrapping_mul(nonce as u128 + 1)
                                .wrapping_add(KEY ^ (MAGIC as u128).rotate_left(7))
                                .wrapping_sub((RATIO as u128).rotate_right(11));
    *internal_seed &= MASK_128;
    (a, b, c, d)}

pub fn permute_1_internal(data: &mut Vec<u8>, internal_seed: &mut u128) {
    if data.is_empty() { return; }
    let seed_low = (*internal_seed & MASK_64 as u128) as u64;
    let seed_high = ((*internal_seed >> 64) & MASK_64 as u128) as u64;
    for i in 0..data.len() {
        let current_byte = data[i] as u64;
        let xored_byte = current_byte ^ seed_low.wrapping_add(i as u64).wrapping_mul(RATIO);
        data[i] = (xored_byte % 256) as u8;
        *internal_seed = internal_seed.wrapping_add(current_byte as u128).rotate_left((i % 128) as u32);}
    *internal_seed = internal_seed.wrapping_mul(seed_high as u128).wrapping_add(seed_low as u128).wrapping_add(KEY).wrapping_sub(CONST as u128);
    *internal_seed &= MASK_128;}

pub fn permute_2_internal(data: &mut Vec<u8>, internal_seed: &mut u128) {
    if data.is_empty() { return; }
    let seed_low = (*internal_seed & MASK_64 as u128) as u64;
    let seed_high = ((*internal_seed >> 64) & MASK_64 as u128) as u64;
    for i in 0..data.len() {
        let j = (i.wrapping_add(seed_low as usize)).wrapping_add((seed_high % 32) as usize).wrapping_mul(MAGIC as usize) % data.len();
        data.swap(i, j);
        *internal_seed = internal_seed.wrapping_sub(data[i] as u128).rotate_right((data[j] % 128) as u32);}
    *internal_seed = internal_seed.wrapping_add(seed_high as u128).wrapping_mul(seed_low as u128).wrapping_sub(KEY ^ (RATIO as u128));
    *internal_seed &= MASK_128;}

pub fn z_func_factory_internal(binary_input: &[u8], _key_input: &[u8]) -> BitGenerator {
    let mut internal_csprng_state: u128 = KEY ^ CONST as u128;
    if !binary_input.is_empty() {
        for chunk in binary_input.chunks(16) {
            let mut chunk_val: u128 = 0;
            for (i, &byte) in chunk.iter().enumerate() {
                chunk_val |= (byte as u128) << (8 * i);}
            internal_csprng_state ^= chunk_val.wrapping_mul(RATIO as u128);}}
    internal_csprng_state &= MASK_128;
    let mut counter: u64 = 0;
    let mut current_output_block: Vec<u8> = Vec::new();
    let mut block_idx: usize = 0;
    Box::new(move || {
        if block_idx >= current_output_block.len() {
            current_output_block.clear();
            block_idx = 0;
            internal_csprng_state = internal_csprng_state.wrapping_add(counter as u128).wrapping_mul(KEY);
            internal_csprng_state &= MASK_128;
            let mut derived_material_state = internal_csprng_state;
            for i in 0..16 {
                derived_material_state = derived_material_state.wrapping_add(counter as u128).wrapping_mul(KEY);
                derived_material_state ^= (derived_material_state >> 64).wrapping_mul(RATIO as u128).wrapping_add(MAGIC as u128);
                derived_material_state = derived_material_state.rotate_left(17 + (i % 7) as u32);
                derived_material_state = derived_material_state.wrapping_add(derived_material_state.rotate_right(31 + (i % 9) as u32));
                derived_material_state ^= (KEY << 32).wrapping_add(CONST as u128);
                derived_material_state &= MASK_128;}
            let mut derived_u64s = Vec::with_capacity(13);
            for i in 0..13 {
                let current_derived_val = (derived_material_state & MASK_64 as u128) as u64;
                derived_u64s.push(current_derived_val);
                derived_material_state = derived_material_state.wrapping_add(current_derived_val as u128).wrapping_mul(RATIO as u128);
                derived_material_state = derived_material_state.rotate_left((i % 12 + 1) as u32);
                derived_material_state ^= (KEY >> (i % 2 * 64)) as u128 ^ (CONST as u128);
                derived_material_state = derived_material_state.wrapping_mul(RATIO as u128).wrapping_add(MAGIC as u128);
                derived_material_state &= MASK_128;}
            let mut derived_idx = 0;
            let mut p_array: [u64; 5] = [0; 5];
            for i in 0..5 {
                p_array[i] = derived_u64s[derived_idx]
                    .wrapping_add(COEFFS[i % 5])
                    .wrapping_mul(MAGIC);
                p_array[i] |= 1;
                const MIN_MODULUS: u64 = 1 << 27;
                if p_array[i] < MIN_MODULUS {
                    p_array[i] = p_array[i].wrapping_add(MIN_MODULUS);
                    p_array[i] |= 1;}
                derived_idx += 1;}
            let mut main_state_arr: [u64; 8] = [0; 8];
            for i in 0..8 {
                main_state_arr[i] = derived_u64s[derived_idx].wrapping_add(MAGIC).wrapping_add(RATIO);
                derived_idx += 1;}
            debug!("CSPRNG generated p_array: {:?}", p_array);
            debug!("CSPRNG generated main_state_arr: {:?}", main_state_arr);
            let h_in_a = (internal_csprng_state & MASK_64 as u128) as u64;
            let h_in_b = ((internal_csprng_state >> 64) & MASK_64 as u128) as u64;
            let h_in_c = h_in_a.wrapping_add(h_in_b).rotate_left(13).wrapping_add(CONST);
            let h_in_d = h_in_b.wrapping_sub(h_in_a).rotate_right(7).wrapping_sub(MAGIC);
            let (out1, out2, out3, out4) = h_func_internal(
                h_in_a, h_in_b, h_in_c, h_in_d,
                counter,
                &mut internal_csprng_state,
                counter,
                &p_array,
                &main_state_arr);
            current_output_block.extend_from_slice(&out1.to_be_bytes());
            current_output_block.extend_from_slice(&out2.to_be_bytes());
            current_output_block.extend_from_slice(&out3.to_be_bytes());
            current_output_block.extend_from_slice(&out4.to_be_bytes());
            counter = counter.wrapping_add(1);}
        let byte = current_output_block[block_idx];
        block_idx += 1;
        byte})}

pub fn t_func_internal(integer_input: u8) -> u8 {
    let x = integer_input as u16;
    let result = (x.wrapping_mul(COEFFS[0] as u16) ^ (x.rotate_left(ARX_BITS[1 % ARX_BITS.len()])).wrapping_add(CONST as u16)) as u8;
    result}
