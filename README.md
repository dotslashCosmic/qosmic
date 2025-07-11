# qosmic

`qosmic` is a Rust-based(formerly Python) project implementing a 512-bit cryptographic hash function designed for robust data integrity, passwords, and even future plans for public/private key pairs, validation, and certificates. It incorporates various cryptographic primitives, including ARX operations, Feistel-like networks, and elements inspired by Learning With Errors (LWE) for quantum resistance considerations.

## Features

* **512-bit Hash Output**: Produces a 64-byte (512-bit) hexadecimal hash.

* **Password Hashing (KDF)**: Integrates a secure Key Derivation Function (KDF) using PBKDF2-HMAC-Qosmic with a salt and high iteration count, designed for robust password storage. This ensures that even the same password produces different hashes each time, protecting against rainbow table attacks.

* **Cryptographic Primitives**: Utilizes a combination of classical and quantum-inspired techniques for strong diffusion and confusion.

* **Command-Line Interface**: Hash strings or files directly from your terminal.

* **NIST Test Script**: Hashes qosmic against several algorithms and compares them in comprehensive NIST approved tests.

     - Must have qosmic built, and Python =>3.11 with hashlib installed.

* **Performance Note**: Currently slightly slower (~2x) than typical hashing algorithms at ~4 cycles/bit, vs ~1.5 cycles/bit for SHA3_512

* **Hashes for qosmic.exe v0.4.0**:
* qosmic hash: `b265d7795b82160e85996614d419393360c57d938391839f69c9195d3b2c7b43d456db1e22a04b953afe2cfe826594acdda311dbf8d455780aa6c4deb53f6f63`
* sha3_512 hash: `c48dea6bcacbcb5a9b5b9786d8a248af5433d74c50d9d225bed21ed069c54b16e5db8c4935c844a61345b7ba333887bea20bd65ce55d9a5515634adbb6f5aca4`
* sha_256 hash: `77fc6caa5fbd7a32ba858dd9279640fe2a3d58b81a3887b28cb6480ee974fac5`

## Installation

To build the `qosmic` executable, ensure you have Rust and Cargo dependencies installed. Then, navigate to the project root directory and run:

`cargo build --release`

This command compiles the project in release mode for optimized performance. The executable will be generated at `target/release/qosmic`.

## Usage
You can hash data by providing either a string or a file as input. Optional logging flags and a persistent key option are available for more detailed output and customizable hashing.

### Basic Syntax
`cargo run --release [--debug|--info] (--interactive | (-f|-s) (file_path|string_input) | --password <password_input>) [--key <key_input>]`

### Arguments

* `--password <password_input>`: Hash the provided password using the integrated Key Derivation Function (PBKDF2-HMAC-Qosmic). This mode generates a salt and performs multiple iterations for secure password storage.

* `--interactive`: Run the application in interactive mode. In this mode, you can continuously input strings to be hashed. You can also set a persistent key by typing `--key <your_key_here>`.

* `-f <file_path>`: Hash the content of the specified file. (Only for non-interactive mode)

* `-s <string_input>`: Hash the provided string. For multi-word strings, enclose the string in double quotes (e.g., "Your text here"). (Only for non-interactive mode)

* `--key <key_input>`: (Optional) Provide a user-defined key for hashing. This key will be mixed into the internal state for stronger customization. For multi-word keys, enclose the key in double quotes (e.g., "my secret key").

* `--debug`: (Optional) Enable debug-level logging for verbose internal process output.

* `--info`: (Optional) Enable info-level logging for general information during execution (e.g., S-Box generation time).

* `--batch-file <file_path>`: Process lines from the specified file. Each line in the file will be treated as a separate input string to be hashed, and the corresponding hash will be printed to standard output. This mode is optimized for performance in batch processing.

* `--version`: Display version information and exit.

* `--help`: Display this help message and exit.

### Examples
**Hashing a string:**

`cargo run --release -- -s "password"`

`qosmic Hash: 690ac209...`

**Hashing a file with info logging (slower):**

`cargo run --release -- --info -f path/to/data.bin`

**Running in interactive mode:**

`cargo run --release -- --interactive`

**Running in batch mode:**

`qosmic --batch-file nonce_list.txt -o hex`

**Normal string hash:**

`cargo run --release -- -s "sensitive data" --key "my secret phrase"`

**Hashing a password:**

`cargo run --release -- --password "mySecurePassword123!"`

## **Cryptographic Test Results:**

* Total successful/attempts: 1000000/1000000

* Elapsed time: 7.69 seconds

* Hashes per second: 130080.20 hashes/sec

* Total bits collected for statistical tests: 512000000 (Each hash is 512 bits)

**Avalanche Effect Test**:

* *EXCELLENT* (100%) (Avg Bit Diff: 50.00%, Ideal: 50.00%, N=999999 comparisons)

**Simple Collision Check Test** (from generated attempts):

* *EXCELLENT* (100%) (No collisions found among generated hashes. Ideal: No Collisions.)

**Monobit Test** (Frequency Test):

* *EXCELLENT* (100%) (X^2 = 0.00, NIST Pass: X^2 < 6.635, df=1)

**Runs Test**:

* *EXCELLENT* (99%) (Total Runs: 256002950, Z-score: 0.26, NIST Pass: Z-score in [-2.576, 2.576], N=512000000)

**Longest Run of Ones Test**:

* *GOOD* (98%) (Longest 1s (overall): 28, Longest 0s (overall): 29, NIST Pass Range: [16 - 52], Total Bits N=512000000)

**Poker Test**:

* *GOOD* (99%) (m=4, X^2 = 5.03, NIST Pass: X^2 < 30.578, df=15, N=512000000)

**Serial Test** (Overlapping Bit Patterns):

* *GOOD* (99%) (Component tests: (m=3, Delta_m): X^2 = 1.57, NIST Pass: X^2 < 18.475, df=7; (m=3, Delta_m-2): X^2 = 0.07, NIST Pass: X^2 < 11.345, df=3)

**Block Frequency Test**:

* *GOOD* (95%) (M=64, X^2 = 7999940.88, NIST Pass: X^2 < 8009306.940, df=8000000, N=512000000)

**Cumulative Sums Test**:

* *EXCELLENT* (99%) (Max Excursion: 21175.00, Z-score: 0.25, NIST Pass: Z-score in [-2.576, 2.576], N=512000000)

## License

This project is licensed under the GNU GPLv3 License.
