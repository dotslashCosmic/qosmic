// src/components.rs
use crate::constants::*;
use crate::qosmic;
use crate::utils::{self, key_as_u512, key_as_u128, key_as_u32, key_as_u64};
use log::debug;

pub type BitGenerator = Box<dyn FnMut() -> u8>;

#[inline]
pub fn v_func_internal(mut x: u64, nonce: u64, p_array_val: &[u64; 5]) -> u64 {
    debug!("v_func_internal: x_initial={:x}, nonce={:x}, p_array_val={:?}", x, nonce, p_array_val);
    for i in 0..24 {
        x = x.wrapping_add(nonce);
        debug!("v_func_internal: round={} x_after_add_nonce={:x}", i, x);
        let rot_amount_1 = 17u32.wrapping_add((nonce as u32).wrapping_mul(i as u32) % 64);
        x = x.wrapping_add(x.rotate_left(rot_amount_1));
        debug!("v_func_internal: round={} x_after_rot1_add={:x}", i, x);
        let rot_amount_2 = 11u32.wrapping_add((i as u32).wrapping_mul((MAGIC % 64) as u32));
        x ^= x.rotate_left(rot_amount_2);
        debug!("v_func_internal: round={} x_after_rot2_xor={:x}", i, x);
        let rot_amount_3 = 7u32.wrapping_add((i as u32).wrapping_mul((RATIO % 64) as u32));
        x ^= x.rotate_left(rot_amount_3);
        debug!("v_func_internal: round={} x_after_rot3_xor={:x}", i, x);
        x = x.wrapping_add(MAGIC);
        debug!("v_func_internal: round={} x_after_add_magic={:x}", i, x);
        let rot_amount_4 = 5u32.wrapping_add((i as u32).wrapping_mul(key_as_u32() % 64));
        x ^= x.rotate_left(rot_amount_4);
        debug!("v_func_internal: round={} x_after_rot4_xor={:x}", i, x);
        x = x.wrapping_add(RATIO ^ (key_as_u64()));
        debug!("v_func_internal: round={} x_final_in_loop={:x}", i, x);}
    let exponent_val = COEFFS[0] | 0x10001;
    let modulus_val_p0 = p_array_val[0];
    debug!("v_func_internal: exponent_val={:x}, modulus_val_p0={:x}", exponent_val, modulus_val_p0);
    if modulus_val_p0 < 2 {
        debug!("Warning: v_func_internal received modulus_val_p0 < 2. Using fallback Q_MOD for pow_mod.");
        return utils::pow_mod(x, exponent_val, Q_MOD);}
    let result = utils::pow_mod(x, exponent_val, modulus_val_p0);
    debug!("v_func_internal: final_result={:x}", result);
    result}

#[inline]
pub fn w_func_internal(a: u64, b: u64, internal_seed: &mut u128) -> u64 {
    debug!("w_func_internal: a={:x}, b={:x}, internal_seed_initial={:x}", a, b, *internal_seed);
    let mut result = a.wrapping_add(b).wrapping_mul(RATIO);
    debug!("w_func_internal: result_after_add_mul_ratio={:x}", result);
    let seed_low = (*internal_seed & MASK_64 as u128) as u64;
    let seed_high = ((*internal_seed >> 64) & MASK_64 as u128) as u64;
    debug!("w_func_internal: seed_low={:x}, seed_high={:x}", seed_low, seed_high);
    result = result.wrapping_add(seed_low).rotate_left(13);
    debug!("w_func_internal: result_after_add_seedlow_rot={:x}", result);
    result ^= seed_high.wrapping_sub(MAGIC);
    debug!("w_func_internal: result_after_xor_seedhigh_sub_magic={:x}", result);
    result = result.rotate_left(7).wrapping_add(b);
    debug!("w_func_internal: result_after_rot7_add_b={:x}", result);
    result ^= result >> 13;
    debug!("w_func_internal: result_after_xor_rshift13={:x}", result);
    result = result.wrapping_mul(MAGIC).wrapping_add(CONST);
    debug!("w_func_internal: result_after_mul_magic_add_const={:x}", result);
    result ^= (a.wrapping_sub(b)).rotate_right(19).wrapping_sub(key_as_u64());
    debug!("w_func_internal: result_after_xor_ab_rot_sub_key={:x}", result);
    result = result.wrapping_add(RATIO).rotate_left(5);
    debug!("w_func_internal: result_after_add_ratio_rot5={:x}", result);
    result ^= (result >> 17).wrapping_mul(CONST);
    debug!("w_func_internal: result_after_xor_rshift17_mul_const={:x}", result);
    *internal_seed = internal_seed
        .wrapping_add(result as u128)
        .wrapping_mul(b as u128 | 1)
        .wrapping_add(key_as_u128() ^ ((*internal_seed >> 3) as u128) ^ (MAGIC as u128));
    debug!("w_func_internal: internal_seed_before_mask={:x}", *internal_seed);
    *internal_seed &= MASK_128;
    debug!("w_func_internal: internal_seed_final={:x}", *internal_seed);
    result}

#[inline]
pub fn d_func_internal(x: u64, y: u64, z: u64, internal_seed: &mut u128, main_state_arr: &[u64; 8]) -> u64 {
    debug!("d_func_internal: x={:x}, y={:x}, z={:x}, internal_seed_initial={:x}, main_state_arr={:?}", x, y, z, *internal_seed, main_state_arr);
    let mut output = x.wrapping_add(y).wrapping_add(z).wrapping_mul(RATIO);
    debug!("d_func_internal: output_after_add_mul_ratio={:x}", output);
    output ^= (*internal_seed & MASK_64 as u128) as u64;
    debug!("d_func_internal: output_after_xor_seedlow={:x}", output);
    output = output.rotate_right(5).wrapping_add(CONST);
    debug!("d_func_internal: output_after_rot5_add_const={:x}", output);
    for k in 0..4 {
        debug!("d_func_internal: loop_k={}", k);
        let state_val_k = main_state_arr[k % 8];
        debug!("d_func_internal: state_val_k={:x}", state_val_k);
        output ^= state_val_k;
        debug!("d_func_internal: output_after_xor_state_val_k={:x}", output);
        output = output.wrapping_mul(RATIO.wrapping_add(state_val_k));
        debug!("d_func_internal: output_after_mul_ratio_add_state_val_k={:x}", output);
        output = output.rotate_left(state_val_k.wrapping_rem(63) as u32 + 1);
        debug!("d_func_internal: output_after_rot_left={:x}", output);
        output ^= ((*internal_seed >> (k * 16 % 128)) & MASK_64 as u128) as u64;
        debug!("d_func_internal: output_after_xor_shifted_seed={:x}", output);
        output = output.wrapping_add(MAGIC.rotate_right((k % 64) as u32));
        debug!("d_func_internal: output_after_add_magic_rot={:x}", output);}
    output = output.wrapping_sub(x.wrapping_add(y).wrapping_mul(z)).rotate_right(11);
    debug!("d_func_internal: output_after_sub_mul_rot={:x}", output);
    output ^= ((*internal_seed >> 32) as u64).wrapping_add(CONST);
    debug!("d_func_internal: output_after_xor_shifted_seed_add_const={:x}", output);
    *internal_seed = internal_seed
        .wrapping_add(output as u128)
        .wrapping_mul(main_state_arr[0] as u128 | 1)
        .wrapping_add(main_state_arr[7] as u128)
        .wrapping_sub(key_as_u128() ^ (RATIO as u128));
    debug!("d_func_internal: internal_seed_before_mask={:x}", *internal_seed);
    *internal_seed &= MASK_128;
    debug!("d_func_internal: internal_seed_final={:x}", *internal_seed);
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
    p_array: &[u64; 5],
    main_state_arr: &[u64; 8],
) -> (u64, u64, u64, u64) {
    debug!("h_func_internal: initial a={:x}, b={:x}, c={:x}, d={:x}, seed_h={:x}, internal_seed={:x}, nonce={:x}, p_array={:?}, main_state_arr={:?}", a, b, c, d, seed_h, *internal_seed, nonce, p_array, main_state_arr);
    a = a.wrapping_add(seed_h).wrapping_mul(RATIO);
    debug!("h_func_internal: a_after_init_transform={:x}", a);
    b = b.wrapping_sub(nonce).rotate_left(ARX_BITS[1]);
    debug!("h_func_internal: b_after_init_transform={:x}", b);
    c = c.wrapping_add(CONST).rotate_right(ARX_BITS[2]);
    debug!("h_func_internal: c_after_init_transform={:x}", c);
    d = d.wrapping_sub(MAGIC).wrapping_add(nonce);
    debug!("h_func_internal: d_after_init_transform={:x}", d);
    for i in 0..4 {
        debug!("h_func_internal: round={} loop_start a={:x}, b={:x}, c={:x}, d={:x}", i, a, b, c, d);
        a = qosmic::v_func(a, nonce.wrapping_add(i as u64), p_array);
        debug!("h_func_internal: round={} a_after_v_func={:x}", i, a);
        b = qosmic::w_func(b, a.wrapping_add(i as u64), internal_seed);
        debug!("h_func_internal: round={} b_after_w_func={:x}", i, b);
        c = qosmic::d_func(c, b.wrapping_add(i as u64), a.wrapping_add(i as u64), internal_seed, main_state_arr);
        debug!("h_func_internal: round={} c_after_d_func={:x}", i, c);
        d = d.wrapping_add(c).rotate_left(ARX_BITS[i % 8].wrapping_add(nonce as u32));
        debug!("h_func_internal: round={} d_after_add_c_rot={:x}", i, d);
        (a, b, c, d) = (
            b ^ c.wrapping_add(d).rotate_left(ARX_BITS[(i + 1) % 8]),
            d.wrapping_add(a).wrapping_mul(RATIO),
            a.rotate_left(13).wrapping_add(b).wrapping_sub(MAGIC),
            b.wrapping_mul(key_as_u64() ^ (nonce.rotate_right(i as u32 % 64))) ^ c.rotate_left(ARX_BITS[(i + 2) % 8]),);
        debug!("h_func_internal: round={} loop_end a={:x}, b={:x}, c={:x}, d={:x}", i, a, b, c, d);}
    let seed_part_a = ((*internal_seed).wrapping_add(key_as_u128()) & MASK_64 as u128) as u64;
    let seed_part_b = (((*internal_seed).wrapping_mul(RATIO as u128)) >> 64 & MASK_64 as u128) as u64;
    debug!("h_func_internal: seed_part_a={:x}, seed_part_b={:x}", seed_part_a, seed_part_b);
    a ^= seed_part_a.rotate_left(7);
    debug!("h_func_internal: a_after_seed_part_a_xor_rot={:x}", a);
    b = b.wrapping_add(seed_part_b.rotate_left(3)).wrapping_mul(CONST);
    debug!("h_func_internal: b_after_seed_part_b_add_mul_const={:x}", b);
    c = c.wrapping_sub(seed_part_a.rotate_right(11)).wrapping_add(MAGIC);
    debug!("h_func_internal: c_after_seed_part_a_sub_add_magic={:x}", c);
    d = d.wrapping_mul(seed_part_b.rotate_right(19)).wrapping_sub(RATIO);
    debug!("h_func_internal: d_after_seed_part_b_mul_sub_ratio={:x}", d);
    a = a.rotate_left(13).wrapping_add(b);
    debug!("h_func_internal: a_final_mix1={:x}", a);
    b = b.rotate_right(17).wrapping_add(c);
    debug!("h_func_internal: b_final_mix1={:x}", b);
    c = c.wrapping_add(d).rotate_left(19);
    debug!("h_func_internal: c_final_mix1={:x}", c);
    d = d.wrapping_add(a).rotate_right(23);
    debug!("h_func_internal: d_final_mix1={:x}", d);
    *internal_seed = internal_seed
        .wrapping_add(a as u128)
        .wrapping_mul(b as u128)
        .wrapping_add(c as u128)
        .wrapping_sub(d as u128)
        .wrapping_add(CONST as u128)
        .wrapping_mul(nonce as u128 + 1)
        .wrapping_add(key_as_u128() ^ (MAGIC as u128).rotate_left(7))
        .wrapping_sub((RATIO as u128).rotate_right(11));
    debug!("h_func_internal: internal_seed_before_mask={:x}", *internal_seed);
    *internal_seed &= MASK_128;
    debug!("h_func_internal: internal_seed_final={:x}", *internal_seed);
    (a, b, c, d)}

pub fn permute_1_internal(data: &mut Vec<u8>, internal_seed: &mut u128) {
    debug!("permute_1_internal: data_initial={:?}, internal_seed_initial={:x}", data, *internal_seed);
    if data.is_empty() {
        debug!("permute_1_internal: data is empty, returning.");
        return;}
    let seed_low = (*internal_seed & MASK_64 as u128) as u64;
    let seed_high = ((*internal_seed >> 64) & MASK_64 as u128) as u64;
    debug!("permute_1_internal: seed_low={:x}, seed_high={:x}", seed_low, seed_high);
    for i in 0..data.len() {
        let current_byte = data[i] as u64;
        debug!("permute_1_internal: index={}, current_byte={:x}", i, current_byte);
        let xored_byte = current_byte ^ seed_low.wrapping_add(i as u64).wrapping_mul(RATIO);
        data[i] = (xored_byte % 256) as u8;
        debug!("permute_1_internal: index={}, data_after_xor={:x}", i, data[i]);
        *internal_seed = internal_seed.wrapping_add(current_byte as u128).rotate_left((i % 128) as u32);
        debug!("permute_1_internal: index={}, internal_seed_in_loop={:x}", i, *internal_seed);}
    *internal_seed = internal_seed
        .wrapping_mul(seed_high as u128)
        .wrapping_add(seed_low as u128)
        .wrapping_add(key_as_u128())
        .wrapping_sub(CONST as u128);
    debug!("permute_1_internal: internal_seed_before_mask_final={:x}", *internal_seed);
    *internal_seed &= MASK_128;
    debug!("permute_1_internal: internal_seed_final={:x}, data_final={:?}", *internal_seed, data);}

pub fn permute_2_internal(data: &mut Vec<u8>, internal_seed: &mut u128) {
    debug!("permute_2_internal: data_initial={:?}, internal_seed_initial={:x}", data, *internal_seed);
    if data.is_empty() {
        debug!("permute_2_internal: data is empty, returning.");
        return;}
    let seed_low = (*internal_seed & MASK_64 as u128) as u64;
    let seed_high = ((*internal_seed >> 64) & MASK_64 as u128) as u64;
    debug!("permute_2_internal: seed_low={:x}, seed_high={:x}", seed_low, seed_high);
    for i in 0..data.len() {
        let j = (i.wrapping_add(seed_low as usize)).wrapping_add((seed_high % 32) as usize).wrapping_mul(MAGIC as usize) % data.len();
        debug!("permute_2_internal: index={}, swap_index={}", i, j);
        data.swap(i, j);
        debug!("permute_2_internal: data_after_swap={:?}", data);
        *internal_seed = internal_seed.wrapping_sub(data[i] as u128).rotate_right((data[j] % 128) as u32);
        debug!("permute_2_internal: index={}, internal_seed_in_loop={:x}", i, *internal_seed);}
    *internal_seed = internal_seed
        .wrapping_add(seed_high as u128)
        .wrapping_mul(seed_low as u128)
        .wrapping_sub(key_as_u128() ^ (RATIO as u128));
    debug!("permute_2_internal: internal_seed_before_mask_final={:x}", *internal_seed);
    *internal_seed &= MASK_128;
    debug!("permute_2_internal: internal_seed_final={:x}, data_final={:?}", *internal_seed, data);}

#[inline]
pub fn t_func_internal() -> [u128; 4] {
    debug!("x_func_internal: called");
    let key_512 = key_as_u512();
    let mut state = key_512;
    debug!("x_func_internal: initial state={:?}", state);
    for i in 0..4 {
        state[0] = state[0].wrapping_add(state[1]).rotate_left(ARX_BITS[i % 8].wrapping_add(7) as u32);
        state[1] = state[1].wrapping_mul(state[2]).wrapping_add(MAGIC as u128);
        state[2] = state[2].wrapping_add(state[3]).rotate_right(ARX_BITS[(i + 1) % 8].wrapping_add(11) as u32);
        state[3] = state[3].wrapping_mul(state[0]).wrapping_sub(RATIO as u128);
        debug!("x_func_internal: round={} state_after_main_ops={:?}", i, state);
        state[0] = state[0].wrapping_mul(key_512[i % 4]).wrapping_add(CONST as u128);
        state[1] = state[1].wrapping_add(key_512[(i + 1) % 4]).rotate_left(COEFFS[i % 5] as u32);
        state[2] = state[2].wrapping_mul(key_512[(i + 2) % 4]).wrapping_sub(MAGIC as u128);
        state[3] = state[3].wrapping_add(key_512[(i + 3) % 4]).rotate_right(COEFFS[(i + 1) % 5] as u32);
        debug!("x_func_internal: round={} state_after_key_mix={:?}", i, state);}
    state}
