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


**Performance Snapshot:**
Hashes per second: 143,657.53 hashes/sec (based on 1,000,000 iterations on 16 CPU cores)

Avalanche Effect: Average Bit Difference of exactly 50.00%, matching the ideal for cryptographic hashes.

Simple Collision Check: No collisions found among 1,000,000 generated hashes.

Monobit Test (Frequency Test): X2 = 0.03 (well within NIST Pass criteria).

Runs Test: Z-score = 0.35 (comfortably within NIST Pass criteria).

Longest Run of Ones: 27

Longest Run of Zeros: 25

Poker Test: X2 = 11.34 (within NIST Pass criteria).

Serial Test (Overlapping Bit Patterns):(m=3, Delta_m): X2 = 0.46(m=3, Delta_m-2): X2 = 0.15

Block Frequency Test: X2 = 8001155.25 (within NIST Pass criteria).

Cumulative Sums Test: Z-score = -0.27 (within NIST Pass criteria).

## License

This project is licensed under the GNU GPLv3 License.
