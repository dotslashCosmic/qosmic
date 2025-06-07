# qosmic_512

`qosmic_512` is a Rust-based(formerly Python) project implementing a 512-bit cryptographic hash function designed for robust data integrity. It incorporates various cryptographic primitives, including ARX operations, Feistel-like networks, and elements inspired by Learning With Errors (LWE) for quantum resistance considerations.

## Features

* **512-bit Hash Output**: Produces a 64-byte (512-bit) hexadecimal hash.

* **Cryptographic Primitives**: Utilizes a combination of classical and quantum-inspired techniques for strong diffusion and confusion.

* **Command-Line Interface**: Hash strings or files directly from your terminal.

* **Performance Note**: Currently slightly slower (~2x) than typical hashing algorithms at ~4 cycles/bit, vs ~1.5 cycles/bit for SHA3_512

* **Hashes for qosmic_512.exe v0.2.6**:
* qosmic_512 hash: `3a85861b83876887d7c3375155c0390461114b9ba6a83ede48990039be43a0e5e8917d3871082f642aaa92307d41ecb9a38a65b149019794fc94c7df6a33c87f`
* sha3_512 hash: `1fd9950d4aa91bcbf03867507a2e7bd9090f1c21c43549fe5d1edabbda8671adc267707224ba1c02c8892ad4556bae0bd56a5c3f3f6ebd064e0db4a1e60ee39d`

## Installation

To build the `qosmic_512` executable, ensure you have Rust and Cargo/dependencies installed. Then, navigate to the project root directory and run:

`cargo build --release`

This command compiles the project in release mode for optimized performance. The executable will be generated at `target/release/qosmic_512`.

## Usage

You can hash data by providing either a string or a file as input. Optional logging flags are available for more detailed output.

### Basic Syntax

`cargo run --release [--debug|--info] (-f|-s) (file_path|string_input)`

### Arguments

* `-f <file_path>`: Hash the content of the specified file.

* `-s <string_input>`: Hash the provided string. For multi-word strings, enclose the string in double quotes (e.g., `"Your text here"`).

* `--debug`: (Optional) Enable debug-level logging for verbose internal process output.

* `--info`: (Optional) Enable info-level logging for general information during execution (e.g., S-Box generation time).

### Examples

**Hashing a string:**

`cargo run --release -- -s "password"`

`qosmic_512 Hash: 690ac2095f55da52e999e3715d7c9604f9269887f2ed3f92625f6306f9ceab9e1237ddf0b755063f00e396459d949d6909da021184d2d83e58bdb1981b5f5a4a`

**Hashing a file with info logging(slower):**

`cargo run --release -- --info -f path/to/data.bin`

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
