# qosmic_512

`qosmic_512` is a Rust-based(formerly Python) project implementing a 512-bit cryptographic hash function designed for robust data integrity. It incorporates various cryptographic primitives, including ARX operations, Feistel-like networks, and elements inspired by Learning With Errors (LWE) for quantum resistance considerations.

## Features

* **512-bit Hash Output**: Produces a 64-byte (512-bit) hexadecimal hash.

* **Cryptographic Primitives**: Utilizes a combination of classical and quantum-inspired techniques for strong diffusion and confusion.

* **Command-Line Interface**: Hash strings or files directly from your terminal.

* **Performance Note**: Currently slightly slower (~2x) than typical hashing algorithms at ~4 cycles/bit, vs ~1.5 cycles/bit for SHA3_512

* **Hashes for qosmic_512.exe v0.2.7**:
* qosmic_512 hash: `89a606f899273fd292834146db78c90f285be14af5d4de77dcc1f2962216f27ef4bba6ff02218b6f4d2e6d7194fe5d6ba627a428ab2d5c8bd8e5be36700a195e`
* sha3_512 hash: `0ddba35e024b7648e6129caa6889829cc752dd5fd7b48bc881d6f0509bcdf5d507110c8e4b4f0df01b62dad086a531e5c871b2ed37fe8ed8c834f63df1291e17`
* sha_256 hash: `7d3747399daa7d3c2380b173450b47efb1a7a976d54b1ca76332074d72388f29`

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
