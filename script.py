import subprocess
import sys
import os
import time
import math
from collections import Counter
import numpy as np
import hashlib
import multiprocessing

_qosmic_executable_path = "target/release/qosmic.exe" if sys.platform == "win32" else "target/release/qosmic"

def _initialize_qosmic_process_for_worker(qosmic_executable_path: str):
    if not os.path.exists(qosmic_executable_path) and not any(os.path.exists(os.path.join(path, qosmic_executable_path)) for path in os.environ["PATH"].split(os.pathsep)):
        print(f"Error: '{qosmic_executable_path}' for Qosmic not found. Please ensure it's compiled and in the same directory or in your system's PATH.", file=sys.stderr)
        return None
    try:
        qosmic_process = subprocess.Popen(
            [qosmic_executable_path, "--interactive"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True, 
            bufsize=1 )
        time.sleep(0.1) 
        return qosmic_process
    except Exception as e:
        print(f"Failed to start Qosmic interactive process in worker: {e}", file=sys.stderr)
        return None

def terminate_worker(qosmic_process_instance):
    if qosmic_process_instance:
        try:
            qosmic_process_instance.stdin.write("EXIT\n")
            qosmic_process_instance.stdin.flush()
            qosmic_process_instance.stdin.close()
            qosmic_process_instance.wait(timeout=5)
        except subprocess.TimeoutExpired:
            print(f"Qosmic process in worker did not terminate gracefully, killing it.", file=sys.stderr)
            qosmic_process_instance.kill()
        except Exception as e:
            print(f"Error terminating Qosmic process in worker: {e}", file=sys.stderr)

def hex_to_bin(hex_string: str) -> str:
    try:
        return bin(int(hex_string, 16))[2:].zfill(len(hex_string) * 4)
    except ValueError:
        return ""

def hamming_distance(s1: str, s2: str) -> int:
    max_len = max(len(s1), len(s2))
    s1 = s1.zfill(max_len)
    s2 = s2.zfill(max_len)
    return sum(c1 != c2 for c1, c2 in zip(s1, s2))

def run_hash_algorithm(message: str, algorithm: str, qosmic_process_instance=None) -> str | None:
    message_bytes = message.encode('utf-8')
    if algorithm == "qosmic":
        if qosmic_process_instance is None:
            print("Error: Qosmic interactive process instance not provided.", file=sys.stderr)
            return None
        try:
            qosmic_process_instance.stdin.write(message + "\n")
            qosmic_process_instance.stdin.flush()
            hash_output_hex = qosmic_process_instance.stdout.readline().strip()
            return hash_output_hex if hash_output_hex else None
        except Exception as e:
            print(f"Error communicating with Qosmic process in worker: {e}", file=sys.stderr)
            return None
    elif algorithm == "blake2b":
        return hashlib.blake2b(message_bytes).hexdigest()
    elif algorithm == "blake2s":
        return hashlib.blake2s(message_bytes).hexdigest()
    elif algorithm == "sha3_512":
        return hashlib.sha3_512(message_bytes).hexdigest()
    elif algorithm == "sha256":
        return hashlib.sha256(message_bytes).hexdigest()
    elif algorithm == "sha512":
        return hashlib.sha512(message_bytes).hexdigest()
    elif algorithm == "shake_256":
        return hashlib.shake_256(message_bytes).hexdigest(64)
    else:
        print(f"Error: Unknown hashing algorithm specified: {algorithm}", file=sys.stderr)
        return None

CHI2_CRITICAL_VALUES_ALPHA_01 = {
    1: 6.635, 2: 9.210, 3: 11.345, 4: 13.277, 5: 15.086,
    6: 16.812, 7: 18.475, 8: 20.090, 9: 21.666, 10: 23.209,
    11: 24.725, 12: 26.217, 13: 27.688, 14: 29.141, 15: 30.578,
    16: 32.000, 17: 33.409, 18: 34.805, 19: 36.191, 20: 37.566,
    21: 38.932, 22: 40.289, 23: 41.638, 24: 42.980, 25: 44.314,
    26: 45.642, 27: 46.963, 28: 48.278, 29: 49.588, 30: 50.892,
    40: 63.691, 50: 76.154, 60: 88.379, 70: 100.425, 80: 112.329,
    90: 124.116, 100: 135.807, 120: 159.083, 140: 182.221, 160: 205.270,
    180: 228.252, 200: 251.179, 220: 274.058, 240: 296.897, 260: 319.702,
    280: 342.478, 300: 365.228, 400: 478.431, 500: 591.261, 1000: 1152.015}
Z_CRITICAL_ALPHA_01 = 2.576 

def get_chi_squared_critical_value(df: int, alpha: float = 0.01) -> float:
    if df <= 0:
        return 0.0 
    if alpha != 0.01:
        print(f"Warning: Only hardcoded critical values for alpha=0.01 are available. Using 0.01 for df={df}.", file=sys.stderr)
    if df in CHI2_CRITICAL_VALUES_ALPHA_01:
        return CHI2_CRITICAL_VALUES_ALPHA_01[df]
    elif df > 1000: 
        Z_val = 2.326 
        return df * (1 - (2.0 / (9.0 * df)) + Z_val * math.sqrt(2.0 / (9.0 * df)))**3
    else:
        closest_df = max([k for k in CHI2_CRITICAL_VALUES_ALPHA_01 if k <= df] or [1])
        if closest_df <= df: 
            val = CHI2_CRITICAL_VALUES_ALPHA_01.get(closest_df, closest_df + Z_CRITICAL_ALPHA_01 * math.sqrt(2 * closest_df))
            return val * (df / closest_df) if closest_df != 0 else val 
        else:
            return df + Z_CRITICAL_ALPHA_01 * math.sqrt(2 * df) 

def calculate_chi_squared(observed_counts: dict, expected_frequency: float, df: int) -> float:
    if expected_frequency <= 0: 
        return float('inf') 
    x_squared = 0.0
    for count in observed_counts.values():
        x_squared += ((count - expected_frequency)**2) / expected_frequency
    return x_squared

def get_perfection_percentage_chi2(x_squared: float, df: int, ideal_x_squared: float = 0.0) -> float:
    if df <= 0: return 0.0 
    nist_pass_critical = get_chi_squared_critical_value(df, alpha=0.01)
    if x_squared <= ideal_x_squared:
        return 100.0 
    if x_squared <= nist_pass_critical:
        return 100 - (5 * (x_squared / nist_pass_critical)) 
    else:
        bad_threshold = nist_pass_critical * 5.0 
        if x_squared >= bad_threshold:
            return 0.0
        return 95 * (1 - (x_squared - nist_pass_critical) / (bad_threshold - nist_pass_critical))

def get_perfection_percentage_zscore(abs_z_score: float, bad_z_threshold: float | None = None) -> float:
    if abs_z_score <= 0.0:
        return 100.0
    if bad_z_threshold is None:
        bad_z_threshold = Z_CRITICAL_ALPHA_01 * 5.0 
    if bad_z_threshold <= Z_CRITICAL_ALPHA_01: 
        bad_z_threshold = Z_CRITICAL_ALPHA_01 + 0.1 
    if abs_z_score <= Z_CRITICAL_ALPHA_01:
        return 100 - (5 * (abs_z_score / Z_CRITICAL_ALPHA_01))
    else:
        if abs_z_score >= bad_z_threshold:
            return 0.0
        if (bad_z_threshold - Z_CRITICAL_ALPHA_01) == 0:
            return 0.0 if abs_z_score > Z_CRITICAL_ALPHA_01 else 95.0
        return 95 * (1 - (abs_z_score - Z_CRITICAL_ALPHA_01) / (bad_z_threshold - Z_CRITICAL_ALPHA_01))

def get_perfection_percentage_range(value: float, min_ideal: float, max_ideal: float, max_deviation_for_zero_percent: float) -> float:
    if min_ideal > max_ideal: return 0.0 
    range_center = (min_ideal + max_ideal) / 2.0
    range_half_width = (max_ideal - min_ideal) / 2.0
    if value >= min_ideal and value <= max_ideal:
        if range_half_width == 0: return 100.0 
        deviation_from_center = abs(value - range_center)
        if range_half_width == 0: return 100.0
        return 100 - (5 * (deviation_from_center / range_half_width))
    else:
        deviation_from_closest_boundary = 0.0
        if value < min_ideal:
            deviation_from_closest_boundary = min_ideal - value
        else: 
            deviation_from_closest_boundary = value - max_ideal
        if max_deviation_for_zero_percent <= 0 or (deviation_from_closest_boundary >= max_deviation_for_zero_percent):
            return 0.0 
        return 95 * (1 - (deviation_from_closest_boundary / max_deviation_for_zero_percent))

def avalanche_effect_test(collected_hashes_bin: list[str]) -> tuple[str, int, str]:
    if len(collected_hashes_bin) < 2:
        return "SKIPPED", 0, "Not enough successful hashes to perform avalanche effect test."
    total_bit_differences = 0
    num_comparisons = 0
    hash_length_bits = len(collected_hashes_bin[0])
    for i in range(len(collected_hashes_bin) - 1):
        hash1_bin = collected_hashes_bin[i]
        hash2_bin = collected_hashes_bin[i+1]
        diff = hamming_distance(hash1_bin, hash2_bin)
        total_bit_differences += diff
        num_comparisons += 1
    if num_comparisons == 0 or hash_length_bits == 0:
        return "SKIPPED", 0, "No comparisons made or hash length is zero."
    average_bit_difference = total_bit_differences / num_comparisons
    average_percentage = (average_bit_difference / hash_length_bits) * 100
    ideal_percentage_target = 50.0
    tolerance_excellent = 0.1  
    tolerance_good = 0.5       
    tolerance_ok = 2.0         
    deviation = abs(average_percentage - ideal_percentage_target)
    grade = ""
    if deviation <= tolerance_excellent:
        grade = "EXCELLENT"
    elif deviation <= tolerance_good:
        grade = "GOOD"
    elif deviation <= tolerance_ok:
        grade = "OK"
    else:
        grade = "BAD"
    very_bad_deviation = 5.0 
    if deviation <= tolerance_good:
        if tolerance_good == 0: perfection_percentage = 100.0
        else: perfection_percentage = 100 - (5 * (deviation / tolerance_good))
    elif deviation <= very_bad_deviation:
        if (very_bad_deviation - tolerance_good) == 0: perfection_percentage = 0.0
        else: perfection_percentage = 95 * (1 - (deviation - tolerance_good) / (very_bad_deviation - tolerance_good))
    else:
        perfection_percentage = 0.0
    perfection_percentage = max(0.0, min(100.0, perfection_percentage))
    details = (f"Avg Bit Diff: {average_percentage:.2f}%, Ideal: {ideal_percentage_target:.2f}%, "
               f"N={num_comparisons} comparisons")
    return grade, int(perfection_percentage), details

def monobit_test(bit_string: str) -> tuple[str, int, str]:
    n = len(bit_string)
    if n < 100:
        return "SKIPPED", 0, f"Not enough bits, N={n} < 100."
    num_ones = bit_string.count('1')
    num_zeros = n - num_ones
    observed_counts = {'0': num_zeros, '1': num_ones}
    expected_freq = n / 2.0
    df = 1 
    x_squared = calculate_chi_squared(observed_counts, expected_freq, df)
    nist_pass_critical = get_chi_squared_critical_value(df, alpha=0.01) 
    if x_squared < 0.1: 
        grade = "EXCELLENT"
    elif x_squared <= nist_pass_critical:
        grade = "GOOD" 
    elif x_squared <= nist_pass_critical * 2.0: 
        grade = "OK"
    else:
        grade = "BAD"
    perfection_percentage = get_perfection_percentage_chi2(x_squared, df)
    perfection_percentage = max(0, min(100, int(perfection_percentage)))
    details = (f"X^2 = {x_squared:.2f}, NIST Pass: X^2 < {nist_pass_critical:.3f}, df={df}")
    return grade, perfection_percentage, details

def runs_test(bit_string: str) -> tuple[str, int, str]:
    n = len(bit_string)
    if n < 20: 
        return "SKIPPED", 0, f"Not enough bits, N={n} < 20."
    num_runs = 0
    if n > 0:
        num_runs = 1
        for i in range(1, n):
            if bit_string[i] != bit_string[i-1]:
                num_runs += 1
    expected_runs = n / 2.0
    std_dev_runs = math.sqrt(n) / 2
    if std_dev_runs == 0:
        return "SKIPPED", 0, "Standard deviation is zero, cannot calculate Z-score."
    z_score = (num_runs - expected_runs) / std_dev_runs
    abs_z_score = abs(z_score)
    nist_z_critical = Z_CRITICAL_ALPHA_01 
    if abs_z_score < 0.5: 
        grade = "EXCELLENT"
    elif abs_z_score <= nist_z_critical:
        grade = "GOOD" 
    elif abs_z_score <= nist_z_critical * 1.5:
        grade = "OK"
    else:
        grade = "BAD"
    perfection_percentage = get_perfection_percentage_zscore(abs_z_score)
    perfection_percentage = max(0, min(100, int(perfection_percentage)))
    details = (f"Total Runs: {num_runs:.0f}, Z-score: {z_score:.2f}, "
               f"NIST Pass: Z-score in [{-nist_z_critical:.3f}, {nist_z_critical:.3f}], N={n}")
    return grade, perfection_percentage, details

def longest_run_of_ones_test(collected_hash_data: list[dict]) -> tuple[str, int, str]:
    if not collected_hash_data:
        return "SKIPPED", 0, "No hashes collected for longest run test."
    n = len("".join([item['bin_hash'] for item in collected_hash_data])) 
    if n < 128: 
        return "SKIPPED", 0, f"Not enough bits for overall test, N={n} < 128."
    overall_max_run_ones = 0
    hash_with_longest_ones_info = None 
    overall_max_run_zeros = 0
    hash_with_longest_zeros_info = None 
    for item in collected_hash_data:
        bit_string = item['bin_hash']
        message = item['message']
        hex_hash = item['hex_hash']
        current_run_ones = 0
        max_run_ones_current_hash = 0
        current_run_zeros = 0
        max_run_zeros_current_hash = 0
        for bit in bit_string:
            if bit == '1':
                current_run_ones += 1
                current_run_zeros = 0
            else:
                current_run_zeros += 1
                current_run_ones = 0
            max_run_ones_current_hash = max(max_run_ones_current_hash, current_run_ones)
            max_run_zeros_current_hash = max(max_run_zeros_current_hash, current_run_zeros)
        if max_run_ones_current_hash > overall_max_run_ones:
            overall_max_run_ones = max_run_ones_current_hash
            hash_with_longest_ones_info = {
                'message': message, 
                'hex_hash': hex_hash, 
                'longest_run': overall_max_run_ones}
        if max_run_zeros_current_hash > overall_max_run_zeros:
            overall_max_run_zeros = max_run_zeros_current_hash
            hash_with_longest_zeros_info = {
                'message': message, 
                'hex_hash': hex_hash, 
                'longest_run': overall_max_run_zeros}
    nist_expected_min, nist_expected_max = 0, 0
    if 128 <= n < 6272: 
        nist_expected_min, nist_expected_max = 1, 10
    elif 6272 <= n < 750000: 
        nist_expected_min, nist_expected_max = 4, 25
    else: 
        nist_expected_min, nist_expected_max = 16, 52
    max_abs_deviation_for_zero_percent = 15 
    perfection_ones = get_perfection_percentage_range(overall_max_run_ones, nist_expected_min, nist_expected_max, max_abs_deviation_for_zero_percent)
    perfection_zeros = get_perfection_percentage_range(overall_max_run_zeros, nist_expected_min, nist_expected_max, max_abs_deviation_for_zero_percent)
    overall_perfection_percentage = min(perfection_ones, perfection_zeros)
    is_nist_pass = (nist_expected_min <= overall_max_run_ones <= nist_expected_max) and \
                   (nist_expected_min <= overall_max_run_zeros <= nist_expected_max)
    grade = ""
    if overall_perfection_percentage >= 99:
        grade = "EXCELLENT"
    elif is_nist_pass:
        grade = "GOOD"
    elif overall_perfection_percentage > 50:
        grade = "OK"
    else:
        grade = "BAD"
    details = (f"Longest 1s (overall): {overall_max_run_ones}, Longest 0s (overall): {overall_max_run_zeros}, "
               f"NIST Pass Range: [{nist_expected_min} - {nist_expected_max}], Total Bits N={n}")
    if hash_with_longest_ones_info:
        details += (f"")
    if hash_with_longest_zeros_info:
        details += (f"")
    return grade, int(overall_perfection_percentage), details

def poker_test(bit_string: str, m: int = 4) -> tuple[str, int, str]:
    n = len(bit_string)
    if n < 2**m * 5: 
        return "SKIPPED", 0, f"Not enough bits for m={m}. N={n}, Recommended N >= {2**m * 5}."
    k = n // m 
    if k < 2**m: 
        return "SKIPPED", 0, f"Not enough unique m-bit blocks possible. k={k}, 2^m={2**m}."
    block_counts = Counter()
    for i in range(k):
        block = bit_string[i*m : (i+1)*m]
        block_val = int(block, 2)
        block_counts[block_val] += 1
    expected_freq = k / (2**m)
    df = (2**m) - 1
    x_squared = calculate_chi_squared(block_counts, expected_freq, df)
    if x_squared == float('inf'):
        return "BAD", 0, f"Error calculating X^2 (expected frequency is zero or invalid)."
    nist_pass_critical = get_chi_squared_critical_value(df, alpha=0.01)
    if x_squared < 0.1: 
        grade = "EXCELLENT"
    elif x_squared <= nist_pass_critical:
        grade = "GOOD" 
    elif x_squared <= nist_pass_critical * 2.0:
        grade = "OK"
    else:
        grade = "BAD"
    perfection_percentage = get_perfection_percentage_chi2(x_squared, df)
    perfection_percentage = max(0, min(100, int(perfection_percentage)))
    details = (f"m={m}, X^2 = {x_squared:.2f}, NIST Pass: X^2 < {nist_pass_critical:.3f}, df={df}, N={n}")
    return grade, perfection_percentage, details

def serial_test(bit_string: str, m: int = 3) -> tuple[str, int, str]:
    n = len(bit_string)
    if n < 2**m * 2: 
        return "SKIPPED", 0, f"Not enough bits, N={n} < {2**m * 2} for m={m}."
    bit_string_circular = bit_string + bit_string[:m-1]

    def calculate_block_frequencies(length: int) -> Counter:
        freqs = Counter()
        if length <= 0: return freqs
        for i in range(n):
            block = bit_string_circular[i : i + length]
            freqs[block] += 1
        return freqs

    def calculate_psi_sq(freqs: Counter, block_len: int, total_n: int) -> float:
        if block_len <= 0 or total_n == 0: return 0.0
        sum_of_squares = sum(count**2 for count in freqs.values())
        expected_factor = (2**block_len) / total_n
        return (sum_of_squares * expected_factor) - total_n

    freq_m_blocks = calculate_block_frequencies(m)
    freq_m_1_blocks = calculate_block_frequencies(m-1) if m > 0 else Counter()
    freq_m_2_blocks = calculate_block_frequencies(m-2) if m > 1 else Counter()
    psi_m_sq = calculate_psi_sq(freq_m_blocks, m, n)
    psi_m_1_sq = calculate_psi_sq(freq_m_1_blocks, m - 1, n) if m > 0 else 0.0
    psi_m_2_sq = calculate_psi_sq(freq_m_2_blocks, m - 2, n) if m > 1 else 0.0
    delta_m_val = psi_m_sq - psi_m_1_sq
    delta_m_2_val = psi_m_1_sq - psi_m_2_sq
    df_delta_m = (2**m) - 1
    df_delta_m_2 = (2**(m-1)) - 1 if m > 0 else 0
    perfection_delta_m = 0.0
    grade_delta_m_str = "SKIPPED"
    details_delta_m = "N/A"
    if df_delta_m > 0:
        perfection_delta_m = get_perfection_percentage_chi2(delta_m_val, df_delta_m)
        nist_pass_critical_m = get_chi_squared_critical_value(df_delta_m, alpha=0.01)
        if delta_m_val < 0.1: grade_delta_m_str = "EXCELLENT"
        elif delta_m_val <= nist_pass_critical_m: grade_delta_m_str = "GOOD"
        elif delta_m_val <= nist_pass_critical_m * 2.0: grade_delta_m_str = "OK"
        else: grade_delta_m_str = "BAD"
        details_delta_m = (f"(m={m}, Delta_m): X^2 = {delta_m_val:.2f}, NIST Pass: X^2 < {nist_pass_critical_m:.3f}, df={df_delta_m}")
    else:
        details_delta_m = f"(m={m}, Delta_m): df={df_delta_m} not sufficient for test."
        perfection_delta_m = 100.0 
    perfection_delta_m_2 = 0.0
    grade_delta_m_2_str = "SKIPPED"
    details_delta_m_2 = "N/A"
    if df_delta_m_2 > 0:
        perfection_delta_m_2 = get_perfection_percentage_chi2(delta_m_2_val, df_delta_m_2)
        nist_pass_critical_m2 = get_chi_squared_critical_value(df_delta_m_2, alpha=0.01)
        if delta_m_2_val < 0.1: grade_delta_m_2_str = "EXCELLENT"
        elif delta_m_2_val <= nist_pass_critical_m2: grade_delta_m_2_str = "GOOD"
        elif delta_m_2_val <= nist_pass_critical_m2 * 2.0: grade_delta_m_2_str = "OK"
        else: grade_delta_m_2_str = "BAD"
        details_delta_m_2 = (f"(m={m}, Delta_m-2): X^2 = {delta_m_2_val:.2f}, NIST Pass: X^2 < {nist_pass_critical_m2:.3f}, df={df_delta_m_2}")
    else:
        details_delta_m_2 = f"(m={m}, Delta_m-2): df={df_delta_m_2} not sufficient for test."
        perfection_delta_m_2 = 100.0 
    overall_perfection_percentage = (perfection_delta_m + perfection_delta_m_2) / 2.0 
    grades_order = {"EXCELLENT": 0, "GOOD": 1, "OK": 2, "BAD": 3, "SKIPPED": 4}
    overall_grade_level = max(grades_order.get(grade_delta_m_str, 4), grades_order.get(grade_delta_m_2_str, 4))
    overall_grade_str = [g for g, level in grades_order.items() if level == overall_grade_level][0]
    final_details = f"Component tests: {details_delta_m}; {details_delta_m_2}"
    return overall_grade_str, int(overall_perfection_percentage), final_details

def block_frequency_test(bit_string: str, m: int = 64) -> tuple[str, int, str]:
    n = len(bit_string)
    if n < m:
        return "SKIPPED", 0, f"Not enough bits, N={n} < M={m}."
    num_blocks = n // m
    if num_blocks == 0:
        return "SKIPPED", 0, f"No complete blocks, N={n}, M={m}."
    if m < 20 or num_blocks < 100:
        print(f"Warning: Block Frequency Test (M={m}) parameters might not meet NIST recommendations (M>=20, N/M>=100).", file=sys.stderr)
    chi_squared_sum = 0.0
    for i in range(num_blocks):
        block = bit_string[i*m : (i+1)*m]
        num_ones_in_block = block.count('1')
        chi_squared_sum += (num_ones_in_block - m/2.0)**2
    x_squared = chi_squared_sum / (m / 4.0)
    df = num_blocks 
    nist_pass_critical = get_chi_squared_critical_value(df, alpha=0.01)
    if x_squared < 0.1:
        grade = "EXCELLENT"
    elif x_squared <= nist_pass_critical:
        grade = "GOOD"
    elif x_squared <= nist_pass_critical * 2.0:
        grade = "OK"
    else:
        grade = "BAD"
    perfection_percentage = get_perfection_percentage_chi2(x_squared, df)
    perfection_percentage = max(0, min(100, int(perfection_percentage)))
    details = (f"M={m}, X^2 = {x_squared:.2f}, NIST Pass: X^2 < {nist_pass_critical:.3f}, df={df}, N={n}")
    return grade, perfection_percentage, details

def cumulative_sums_test(bit_string: str) -> tuple[str, int, str]:
    n = len(bit_string)
    if n < 100: 
        return "SKIPPED", 0, f"Not enough bits, N={n} < 100."
    transformed_bits = [1 if bit == '1' else -1 for bit in bit_string]
    cumulative_sum = [0] * (n + 1) 
    for i in range(n):
        cumulative_sum[i+1] = cumulative_sum[i] + transformed_bits[i]
    max_excursion = max(abs(s) for s in cumulative_sum)
    std_dev_max_excursion = math.sqrt((math.pi**2 - 8) / (2 * math.pi)) * math.sqrt(n) 
    if std_dev_max_excursion == 0:
        return "SKIPPED", 0, "Standard deviation of max excursion is zero."
    expected_max_excursion = math.sqrt(2 / math.pi) * math.sqrt(n) 
    z_score = (max_excursion - expected_max_excursion) / std_dev_max_excursion
    abs_z_score = abs(z_score)
    nist_z_critical = Z_CRITICAL_ALPHA_01 
    if abs_z_score < 0.5: 
        grade = "EXCELLENT"
    elif abs_z_score <= nist_z_critical:
        grade = "GOOD" 
    elif abs_z_score <= nist_z_critical * 1.5:
        grade = "OK"
    else:
        grade = "BAD"
    perfection_percentage = get_perfection_percentage_zscore(abs_z_score)
    perfection_percentage = max(0, min(100, int(perfection_percentage)))
    details = (f"Max Excursion: {max_excursion:.2f}, Z-score: {z_score:.2f}, "
               f"NIST Pass: Z-score in [{-nist_z_critical:.3f}, {nist_z_critical:.3f}], N={n}")
    return grade, perfection_percentage, details

def spectral_test(bit_string: str) -> tuple[str, int, str]:
    n = len(bit_string)
    if n < 1024: 
        return "SKIPPED", 0, f"Not enough bits, N={n} < 1024."
    x = np.array([1 if bit == '1' else -1 for bit in bit_string])
    fft_result = np.fft.fft(x)
    magnitudes = np.abs(fft_result[1:n//2])
    T = math.sqrt(2.0 * math.log(1.0 / 0.05) * n)
    num_peaks_below_T = np.sum(magnitudes < T)
    expected_count = (n / 2.0 - 1) * 0.95
    std_dev_count = math.sqrt((n / 2.0 - 1) * 0.95 * 0.05)
    if std_dev_count == 0:
        return "SKIPPED", 0, "Standard deviation of peak count is zero."
    z_score = (num_peaks_below_T - expected_count) / std_dev_count
    abs_z_score = abs(z_score)
    perfection_percentage = get_perfection_percentage_zscore(abs_z_score, bad_z_threshold=1317.66) 
    perfection_percentage = max(0, min(100, int(perfection_percentage)))
    grade = ""
    if perfection_percentage >= 75: 
        grade = "GOOD"
    elif perfection_percentage >= 65: 
        grade = "OK"
    else: 
        grade = "BAD"
    nist_z_critical = Z_CRITICAL_ALPHA_01 
    details = (f"Peaks below T: {num_peaks_below_T:.0f}, Z-score: {z_score:.2f}, "
               f"NIST Pass: Z-score in [{-nist_z_critical:.3f}, {nist_z_critical:.3f}], N={n}, T={T:.2f}\n    The more simulations, the lower the score may get.")
    return grade, perfection_percentage, details

def collision_check_test(collected_hashes_hex: list[str], collected_messages: list[str]) -> tuple[str, int, str]:
    if not collected_hashes_hex:
        return "SKIPPED", 0, "No hashes collected for collision check."
    found_collision = False
    hash_to_messages = {}
    for i, hash_hex in enumerate(collected_hashes_hex):
        message = collected_messages[i]
        if hash_hex in hash_to_messages:
            found_collision = True
            hash_to_messages[hash_hex].append(message)
        else:
            hash_to_messages[hash_hex] = [message]
    if found_collision:
        collision_details = ""
        collision_count = 0
        for hash_val, messages in hash_to_messages.items():
            if len(messages) > 1:
                collision_count += 1
                collision_details += f"\n      - Hash: {hash_val}, Inputs: {len(messages)}"
                if len(collision_details) < 500: 
                    for msg in messages:
                        collision_details += f"\n        - '{msg}'"
                else:
                    collision_details += "\n        ... (too many to list all inputs)"
        full_details = f"Collision(s) found ({collision_count} unique collisions). {collision_details} (Note: Highly unusual for a cryptographic hash function in a small sample, indicates weakness or extreme chance)."
        return "BAD", 0, full_details
    else:
        return "EXCELLENT", 100, "No collisions found among generated hashes. Ideal: No Collisions."

def hash_generation_worker(args):
    algorithm, base_message, start_index, end_index, qosmic_executable_path = args
    worker_hash_data = [] 
    qosmic_proc_instance = None
    if algorithm == "qosmic":
        qosmic_proc_instance = _initialize_qosmic_process_for_worker(qosmic_executable_path)
        if qosmic_proc_instance is None:
            return [] 
    try:
        for i in range(start_index, end_index):
            nonce = str(i)
            current_message = f"{base_message}{nonce}"
            hash_output_hex = run_hash_algorithm(current_message, algorithm, qosmic_proc_instance)
            if hash_output_hex:
                hash_output_bin = hex_to_bin(hash_output_hex)
                worker_hash_data.append({
                    'message': current_message,
                    'hex_hash': hash_output_hex,
                    'bin_hash': hash_output_bin})
    finally:
        if qosmic_proc_instance:
            terminate_worker(qosmic_proc_instance)
    return worker_hash_data

def simulate_hashing_and_test(base_message: str, num_iterations: int):
    ALGORITHMS_TO_TEST = ["qosmic", "blake2b", "sha3_512", "sha256", "sha512", "blake2s", "shake_256"]
    print("Qosmic Hashing Tester and Standard Hash Algorithm Comparison")
    print(f"This script will test the following algorithms: {', '.join(ALGORITHMS_TO_TEST)}.")
    num_cores = multiprocessing.cpu_count()
    print(f"Detected {num_cores} CPU cores. Will use {num_cores} processes for hash generation.")
    for current_algo in ALGORITHMS_TO_TEST:
        print(f"\n{'='*50}")
        print(f"  TESTING ALGORITHM: {current_algo.upper()}")
        print(f"{'='*50}\n")
        print(f"Starting hashing simulation for {num_iterations} iterations using {current_algo}...")
        print(f"Base Message: '{base_message}'")
        all_collected_hash_data = [] 
        start_time = time.time()
        tasks = []
        iterations_per_core = num_iterations // num_cores
        for i in range(num_cores):
            start_idx = i * iterations_per_core
            end_idx = start_idx + iterations_per_core
            if i == num_cores - 1:
                end_idx = num_iterations
            tasks.append((current_algo, base_message, start_idx, end_idx, _qosmic_executable_path))
        with multiprocessing.Pool(processes=num_cores) as pool:
            results_from_workers = pool.map(hash_generation_worker, tasks)
        for worker_data in results_from_workers:
            all_collected_hash_data.extend(worker_data)
        successful_hashes_count = len(all_collected_hash_data)
        end_time = time.time()
        elapsed_time = end_time - start_time
        print(f"\nSimulation Summary for {current_algo.upper()}")
        print(f"Total successful/attempts: {successful_hashes_count}/{num_iterations}")
        print(f"Elapsed time: {elapsed_time:.2f} seconds")
        if successful_hashes_count > 0 and elapsed_time > 0:
            hashes_per_second = successful_hashes_count / elapsed_time
            print(f"Hashes per second: {hashes_per_second:.2f} hashes/sec")
        else:
            print("Could not calculate hashes per second (no successful hashes or zero elapsed time).")
        print("\nCryptographic Test Results")
        all_collected_hashes_hex = [item['hex_hash'] for item in all_collected_hash_data]
        all_collected_hashes_bin = [item['bin_hash'] for item in all_collected_hash_data]
        all_collected_messages = [item['message'] for item in all_collected_hash_data]
        all_bits = "".join(all_collected_hashes_bin)
        if len(all_bits) == 0:
            print(f"\nNot enough bits collected for rigorous statistical testing for {current_algo}. Ensure hashes are being generated.")
            continue
        current_hash_length_bits = len(all_collected_hashes_bin[0]) if all_collected_hashes_bin else 0
        print(f"Total bits collected for statistical tests: {len(all_bits)} (Each hash is {current_hash_length_bits} bits)")
        test_funcs_params = [
            ("Avalanche Effect Test", avalanche_effect_test, [all_collected_hashes_bin]),
            ("Simple Collision Check Test (from generated attempts)", collision_check_test, [all_collected_hashes_hex, all_collected_messages]),
            ("Monobit Test (Frequency Test)", monobit_test, [all_bits]),
            ("Runs Test", runs_test, [all_bits]),
            ("Longest Run of Ones Test", longest_run_of_ones_test, [all_collected_hash_data]), 
            ("Poker Test", poker_test, [all_bits, 4]),
            ("Serial Test (Overlapping Bit Patterns)", serial_test, [all_bits, 3]),
            ("Block Frequency Test", block_frequency_test, [all_bits, 64]),
            ("Cumulative Sums Test", cumulative_sums_test, [all_bits]),
            ("Discrete Fourier Transform (Spectral) Test", spectral_test, [all_bits])]
        for test_name, func, args_list in test_funcs_params:
            print(f"\n{test_name}")
            try:
                grade, perfection, details = func(*args_list)
                if grade == "SKIPPED":
                    print(f"  {grade}: {details}")
                else:
                    print(f"  {grade} ({perfection}%) ({details})")
            except Exception as e:
                print(f"  ERROR running test: {e}", file=sys.stderr)
                print(f"  Test: {test_name}, Args: {args_list}\nException: {e}", file=sys.stderr) 
    print("\nRigorous Testing Complete for All Algorithms")
    print("NOTE: Passing these statistical tests indicates a strong *statistical* resemblance to randomness,")
    print("but does NOT mathematically prove cryptographic security against all forms of attack.")

if __name__ == "__main__":
    BASE_MESSAGE = "password"
    NUM_ITERATIONS = 1_000_000
    simulate_hashing_and_test(BASE_MESSAGE, NUM_ITERATIONS)
