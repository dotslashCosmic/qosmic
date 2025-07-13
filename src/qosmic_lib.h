// src/qosmic_lib.h
#ifndef QOSMIC_LIB_H
#define QOSMIC_LIB_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Hashes input data using the qosmic algorithm and returns a hex-encoded C string.
 * The returned string is allocated by Rust and MUST be freed by calling `qosmic_free_string` to prevent memory leaks.
 * @param input_ptr A pointer to the byte array to be hashed.
 * @param input_len The length of the byte array.
 * @return A pointer to a null-terminated C string containing the hex-encoded hash,
 * or a null pointer if the input pointer was null or an error occurred.
 */
char* qosmic_hash(const uint8_t* input_ptr, size_t input_len);

/**
 * @brief Frees a C string that was allocated by the Rust `qosmic_hash` function.
 * This function must be called to deallocate the memory for the string returned by `qosmic_hash`.
 * It is safe to pass a null pointer to this function.
 * @param s A pointer to the C string to be freed.
 */
void qosmic_free_string(char* s);

#ifdef __cplusplus
}
#endif

#endif // QOSMIC_LIB_H