# qosmic_512

`qosmic_512` is a Rust-based(formerly Python) project implementing a 512-bit cryptographic hash function designed for robust data integrity. It incorporates various cryptographic primitives, including ARX operations, Feistel-like networks, and elements inspired by Learning With Errors (LWE) for quantum resistance considerations.

## Features

* **512-bit Hash Output**: Produces a 64-byte (512-bit) hexadecimal hash.

* **Cryptographic Primitives**: Utilizes a combination of classical and quantum-inspired techniques for strong diffusion and confusion.

* **Command-Line Interface**: Hash strings or files directly from your terminal.

* **Performance Note**: Currently slightly slower (~2x) than typical hashing algorithms at ~4 cycles/bit, vs ~1.5 cycles/bit for SHA3_512

* **Hashes for qosmic_512.exe v0.2.7**:
* qosmic_512 hash: `62a94bec03c870d5526308f2d1be34858893156fefe148d6a891ec29b9ccd45857f3355c2e7075992ec24c8f1e90c6a31045d1b6a03bc0fc751a4bcc50b3bea1`
* sha3_512 hash: `765c198c40dfca0e42f99384e014f945783c85908cd23ec10ca1a426c6cc8ee474800fe42b6eb4c7ea86c2a1631f7478369128939869ce1944816c327c5cd040`
* sha_256 hash: `a05839b92a9dfecc89926dd06a5c5a4b0626ea7e82f1ace58b757545ffdd8ad6`

## Installation

To build the `qosmic_512` executable, ensure you have Rust and Cargo dependencies installed. Then, navigate to the project root directory and run:

`cargo build --release`

This command compiles the project in release mode for optimized performance. The executable will be generated at `target/release/qosmic_512`.

## Usage
You can hash data by providing either a string or a file as input. Optional logging flags and a persistent key option are available for more detailed output and customizable hashing.

### Basic Syntax
`cargo run --release [--debug|--info] (--interactive | (-f|-s) (file_path|string_input)) [--key <key_input>]`

### Arguments
* `--interactive`: Run the application in interactive mode. In this mode, you can continuously input strings to be hashed. You can also set a persistent key by typing KEY <your_key_here>. Type EXIT or QUIT to end the session.

* `-f <file_path>`: Hash the content of the specified file. (Only for non-interactive mode)

* `-s <string_input>`: Hash the provided string. For multi-word strings, enclose the string in double quotes (e.g., "Your text here"). (Only for non-interactive mode)

* `--key <key_input>`: (Optional) Provide a user-defined key for hashing. This key will be mixed into the internal state for stronger customization. For multi-word keys, enclose the key in double quotes (e.g., "my secret key").

* `--debug`: (Optional) Enable debug-level logging for verbose internal process output.

* `--info`: (Optional) Enable info-level logging for general information during execution (e.g., S-Box generation time).

### Examples
**Hashing a string:**

`cargo run --release -- -s "password"`

`qosmic_512 Hash: 690ac209...`

**Hashing a file with info logging (slower):**

`cargo run --release -- --info -f path/to/data.bin`

Running in interactive mode:

`cargo run --release -- --interactive`

`Enter text to hash (or 'KEY <your_key>' to set a key, 'EXIT'/'QUIT' to exit):
hello world
<qosmic_512 Hash: ...>
KEY my_custom_key
Key set for subsequent hashes.
another input
<qosmic_512 Hash: ...>
EXIT
Hashing a string with a key:`

`cargo run --release -- -s "sensitive data" --key "my secret phrase"`





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
