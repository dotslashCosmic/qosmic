// src/components.rs
use num_bigint::ToBigUint;
use crate::constants::*;
use crate::utils::pow_mod_biguint;
use crate::qosmic;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;


pub type BitGenerator = Box<dyn FnMut() -> u8>;

#[inline]
#[target_feature(enable = "sse2")]
pub fn v_func_internal(mut x: u64, nonce: u64, p_array_val: &[u64; 5]) -> u64 {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let mut x_vec = _mm_set_epi64x(0, x as i64);
        let nonce_vec = _mm_set1_epi64x(nonce as i64);

        for _ in 0..32 {
            x_vec = _mm_add_epi64(x_vec, nonce_vec);

            let left_shifted = _mm_slli_epi64(x_vec, 17);
            let right_shifted = _mm_srli_epi64(x_vec, 64 - 17);
            let rotated_x = _mm_or_si128(left_shifted, right_shifted);
            x_vec = _mm_add_epi64(x_vec, rotated_x);


            let left_shifted_xor1 = _mm_slli_epi64(x_vec, 11);
            let right_shifted_xor1 = _mm_srli_epi64(x_vec, 64 - 11);
            let rotated_x_xor1 = _mm_or_si128(left_shifted_xor1, right_shifted_xor1);
            x_vec = _mm_xor_si128(x_vec, rotated_x_xor1);


            let left_shifted_xor2 = _mm_slli_epi64(x_vec, 7);
            let right_shifted_xor2 = _mm_srli_epi64(x_vec, 64 - 7);
            let rotated_x_xor2 = _mm_or_si128(left_shifted_xor2, right_shifted_xor2);
            x_vec = _mm_xor_si128(x_vec, rotated_x_xor2);
        }
        x = _mm_cvtsi128_si64(x_vec) as u64;
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    {
        for _ in 0..32 {
            x = x.wrapping_add(nonce);
            x = x.wrapping_add(x.rotate_left(17));
            x ^= x.rotate_left(11);
            x ^= x.rotate_left(7);
        }
    }
    
    let exponent_val = COEFFS[0] | 0x10001;
    let modulus_val_p0 = p_array_val[0];

    if modulus_val_p0 < 2 {
        return pow_mod_biguint(
            x.to_biguint().unwrap(),
            exponent_val.to_biguint().unwrap(),
            modulus_val_p0.to_biguint().unwrap()
        ).to_u64_digits().first().cloned().unwrap_or(0);
    }

    pow_mod_biguint(
        x.to_biguint().unwrap(),
        exponent_val.to_biguint().unwrap(),
        modulus_val_p0.to_biguint().unwrap()
    ).to_u64_digits().first().cloned().unwrap_or(0)
}

#[inline]
pub fn w_func_internal(a: u64, b: u64, internal_seed: &mut u128) -> u64 {
    let mut result = a.wrapping_add(b);
    
    let seed_low = (*internal_seed & MASK_64 as u128) as u64;
    let seed_high = ((*internal_seed >> 64) & MASK_64 as u128) as u64;

    result = result.wrapping_add(seed_low);
    result ^= seed_high;

    result = result.rotate_left(7);
    result ^= result >> 13;
    result = result.wrapping_mul(MAGIC);

    *internal_seed = internal_seed.wrapping_mul(result as u128).wrapping_add(b as u128).wrapping_add(KEY);
    *internal_seed &= MASK_128;

    result
}

#[inline]
pub fn d_func_internal(x: u64, y: u64, z: u64, internal_seed: &mut u128, main_state_arr: &[u64; 8]) -> u64 {
    let mut output = x.wrapping_add(y).wrapping_add(z);
    
    output ^= (*internal_seed & MASK_64 as u128) as u64;
    output = output.rotate_right(5);

    for &val in main_state_arr.iter() {
        output ^= val;
        output = output.wrapping_mul(RATIO);
    }
    
    *internal_seed = internal_seed.wrapping_add(output as u128).wrapping_mul(main_state_arr[0] as u128).wrapping_add(KEY);
    *internal_seed &= MASK_128;

    output
}

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
    
    a = a.wrapping_add(seed_h);
    b = b.wrapping_sub(nonce);

    a = qosmic::v_func(a, nonce, p_array);
    b = qosmic::w_func(b, a, internal_seed);
    c = qosmic::d_func(c, b, a, internal_seed, main_state_arr);
    d = d.wrapping_add(c).rotate_left(ARX_BITS[0]);

    let seed_part = (*internal_seed & MASK_64 as u128) as u64;
    a ^= seed_part;
    b = b.wrapping_add(seed_part.rotate_left(3));

    a = a.rotate_left(13).wrapping_add(b);
    b = b.rotate_right(17).wrapping_add(c);
    c = c.wrapping_add(d).rotate_left(19);
    d = d.wrapping_add(a).rotate_right(23);

    *internal_seed = internal_seed.wrapping_add(a as u128)
                                 .wrapping_mul(b as u128)
                                 .wrapping_add(c as u128)
                                 .wrapping_sub(d as u128)
                                 .wrapping_add(CONSTANT_H_PLACEHOLDER as u128);
    *internal_seed &= MASK_128;

    (a, b, c, d)
}


pub fn permute_1_internal(data: &[u8], internal_seed: &mut u128) -> Vec<u8> {
    let mut output = data.to_vec();
    if output.is_empty() { return output; }

    let seed_low = (*internal_seed & MASK_64 as u128) as u64;
    let seed_high = ((*internal_seed >> 64) & MASK_64 as u128) as u64;

    for i in 0..output.len() {
        let current_byte = output[i] as u64;
        let xored_byte = current_byte ^ seed_low.wrapping_add(i as u64);
        output[i] = (xored_byte % 256) as u8;
    }

    *internal_seed = internal_seed.wrapping_mul(seed_high as u128).wrapping_add(seed_low as u128).wrapping_add(KEY);
    *internal_seed &= MASK_128;

    output
}

pub fn permute_2_internal(data: &[u8], internal_seed: &mut u128) -> Vec<u8> {
    let mut output = data.to_vec();
    if output.is_empty() { return output; }

    let seed_low = (*internal_seed & MASK_64 as u128) as u64;
    let seed_high = ((*internal_seed >> 64) & MASK_64 as u128) as u64;

    for i in 0..output.len() {
        let j = (i.wrapping_add(seed_low as usize)) % output.len();
        output.swap(i, j);
    }

    *internal_seed = internal_seed.wrapping_add(seed_high as u128).wrapping_mul(seed_low as u128).wrapping_sub(KEY);
    *internal_seed &= MASK_128;

    output
}

pub fn z_func_factory_internal(binary_input: &[u8], _key_input: &[u8]) -> BitGenerator {
    let initial_state_val = if binary_input.is_empty() {
        0u16
    } else if binary_input.len() == 1 {
        binary_input[0] as u16
    } else {
        u16::from_be_bytes(binary_input[0..2].try_into().unwrap_or([0,0]))
    };
    
    let mut current_lfsr_state_u32 = initial_state_val as u32;
    let taps_u32: [u32; 8] = [0, 2, 4, 7, 10, 12, 14, 16];
    let max_tap = 16;

    Box::new(move || {
        let mut feedback_bit: u32 = 0;
        for &tap_pos in &taps_u32 {
            feedback_bit ^= (current_lfsr_state_u32 >> tap_pos) & 1;
        }
        current_lfsr_state_u32 = (current_lfsr_state_u32 >> 1) | (feedback_bit << (max_tap - 1));
        (current_lfsr_state_u32 & 0xFF) as u8
    })
}

pub fn t_func_internal(integer_input: u8) -> u8 {
    let x = integer_input as u16;
    let result = (x.wrapping_mul(COEFFS[0] as u16) ^ (x.rotate_left(ARX_BITS[1]))) as u8;
    result
}
