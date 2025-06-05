# qosmic_512

`qosmic_512` is a Rust-based(formerly Python) project implementing a 512-bit cryptographic hash function designed for robust data integrity. It incorporates various cryptographic primitives, including ARX operations, Feistel-like networks, and elements inspired by Learning With Errors (LWE) for quantum resistance considerations.

## Features

* **512-bit Hash Output**: Produces a 64-byte (512-bit) hexadecimal hash.

* **Cryptographic Primitives**: Utilizes a combination of classical and quantum-inspired techniques for strong diffusion and confusion.

* **Command-Line Interface**: Hash strings or files directly from your terminal.

* **Performance Note**: Currently significantly slower (~2000x) than typical hashing algorithms at ~2900 cycles/bit, vs ~1.5 cycles/bit for SHA3_512
* Runs at ~40μs/hash per chunk of 64 bytes

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


## License

This project is licensed under the GNU GPLv3 License.
