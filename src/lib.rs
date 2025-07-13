// src/lib.rs
#[macro_use]
extern crate lazy_static;

pub mod constants;
pub mod utils;
pub mod primitives;
pub mod components;
pub mod core;
pub mod encode;

pub use core::{get_sbox, hmac_qosmic, qosmic_unkeyed, SBoxType, hash_password, pbkdf2_hmac_qosmic};
pub use utils::is_prime;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use libc::size_t as c_size_t;
use log::info;

#[repr(C)]
pub enum QosmicErrorCode {
    Success = 0,
    NullInput = 1,
    CStringConversionError = 2,
    HexDecodingError = 3,
    MemoryAllocationError = 4,}

/// @param input_ptr A pointer to the byte array to be hashed.
/// @param input_len The length of the byte array.
/// @param output_hash_ptr A pointer to a `char*` where the hex-encoded hash string will be stored.
/// @return A `QosmicErrorCode` indicating success or the type of error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qosmic_hash_unkeyed(
    input_ptr: *const u8,
    input_len: c_size_t,
    output_hash_ptr: *mut *mut c_char,
) -> QosmicErrorCode {
    *output_hash_ptr = std::ptr::null_mut();
    if input_ptr.is_null() {
        info!("qosmic_hash_unkeyed: input_ptr is null.");
        return QosmicErrorCode::NullInput;}
    let input_slice = std::slice::from_raw_parts(input_ptr, input_len as usize);
    info!("qosmic_hash_unkeyed: Received {} bytes for hashing.", input_len);
    let s_box = core::get_sbox();
    let nonce = utils::derive_deterministic_nonce(input_slice);
    let hash_result = core::qosmic_unkeyed(input_slice.to_vec(), 's', s_box, nonce);
    info!("qosmic_hash_unkeyed: Hashing complete. Result length: {}", hash_result.len());
    match CString::new(hash_result) {
        Ok(c_string) => {
            *output_hash_ptr = c_string.into_raw();
            QosmicErrorCode::Success},
        Err(e) => {
            info!("qosmic_hash_unkeyed: Failed to convert hash result to CString: {}", e);
            QosmicErrorCode::CStringConversionError}}}

/// @param key_ptr A pointer to the key byte array.
/// @param key_len The length of the key byte array.
/// @param message_ptr A pointer to the message byte array.
/// @param message_len The length of the message byte array.
/// @param output_hash_ptr A pointer to a `char*` where the hex-encoded HMAC hash string will be stored.
/// @return A `QosmicErrorCode` indicating success or the type of error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qosmic_hmac_qosmic(
    key_ptr: *const u8,
    key_len: c_size_t,
    message_ptr: *const u8,
    message_len: c_size_t,
    output_hash_ptr: *mut *mut c_char,
) -> QosmicErrorCode {
    *output_hash_ptr = std::ptr::null_mut();
    if key_ptr.is_null() || message_ptr.is_null() {
        info!("qosmic_hmac_qosmic: Null key_ptr or message_ptr.");
        return QosmicErrorCode::NullInput;}
    let key_slice = std::slice::from_raw_parts(key_ptr, key_len as usize);
    let message_slice = std::slice::from_raw_parts(message_ptr, message_len as usize);
    info!("qosmic_hmac_qosmic: Received key_len={} and message_len={}", key_len, message_len);
    let hmac_result = core::hmac_qosmic(key_slice, message_slice);
    info!("qosmic_hmac_qosmic: HMAC calculation complete. Result length: {}", hmac_result.len());
    match CString::new(hmac_result) {
        Ok(c_string) => {
            *output_hash_ptr = c_string.into_raw();
            QosmicErrorCode::Success},
        Err(e) => {
            info!("qosmic_hmac_qosmic: Failed to convert HMAC result to CString: {}", e);
            QosmicErrorCode::CStringConversionError}}}

/// @param password_ptr A pointer to the password byte array.
/// @param password_len The length of the password byte array.
/// @param salt_ptr A pointer to the salt byte array.
/// @param salt_len The length of the salt byte array.
/// @param iterations The number of iterations for PBKDF2.
/// @param output_len_requested The desired length of the derived key in bytes.
/// @param derived_key_ptr A pointer to a `uint8_t*` where the derived key bytes will be stored.
/// @param derived_key_actual_len_ptr A pointer to a `size_t` where the actual length of the derived key will be stored.
/// @return A `QosmicErrorCode` indicating success or the type of error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qosmic_pbkdf2_hmac_qosmic(
    password_ptr: *const u8,
    password_len: c_size_t,
    salt_ptr: *const u8,
    salt_len: c_size_t,
    iterations: u32,
    output_len_requested: c_size_t,
    derived_key_ptr: *mut *mut u8,
    derived_key_actual_len_ptr: *mut c_size_t,
) -> QosmicErrorCode {
    *derived_key_ptr = std::ptr::null_mut();
    *derived_key_actual_len_ptr = 0;
    if password_ptr.is_null() || salt_ptr.is_null() {
        info!("qosmic_pbkdf2_hmac_qosmic: Null password_ptr or salt_ptr.");
        return QosmicErrorCode::NullInput;}
    let password_slice = std::slice::from_raw_parts(password_ptr, password_len as usize);
    let salt_slice = std::slice::from_raw_parts(salt_ptr, salt_len as usize);
    info!("qosmic_pbkdf2_hmac_qosmic: Received password_len={}, salt_len={}, iterations={}, output_len_requested={}",
        password_len, salt_len, iterations, output_len_requested);
    let derived_key_vec = core::pbkdf2_hmac_qosmic(
        password_slice,
        salt_slice,
        iterations,
        output_len_requested as usize,);
    let buffer = derived_key_vec.into_boxed_slice();
    let len = buffer.len();
    *derived_key_ptr = Box::into_raw(buffer) as *mut u8;
    *derived_key_actual_len_ptr = len as c_size_t;
    QosmicErrorCode::Success}

/// @param ptr A pointer to the byte array to be freed.
/// @param len The length of the byte array. This *must* be the same length as returned by the allocation function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qosmic_free_bytes(ptr: *mut u8, len: c_size_t) {
    if ptr.is_null() {
        info!("qosmic_free_bytes: Received null pointer, nothing to free.");
        return;}
    let _ = Box::from_raw(std::slice::from_raw_parts_mut(ptr, len as usize));
    info!("qosmic_free_bytes: Byte array freed successfully (ptr: {:?}, len: {}).", ptr, len);}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qosmic_free_string(s: *mut c_char) {
    if s.is_null() {
        info!("qosmic_free_string: Received null pointer, nothing to free.");
        return;}
    let _ = CString::from_raw(s);
    info!("qosmic_free_string: String freed successfully.");}
