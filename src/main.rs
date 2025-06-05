// src/main.rs
use qosmic_512::{qosmic512, get_sbox, is_prime};
use std::{env, fs, io::Read, process};
use qosmic_512::utils::derive_deterministic_nonce;
use log::LevelFilter;
use env_logger::Builder;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut log_level = LevelFilter::Off;

    if args.contains(&"--debug".to_string()) {
        log_level = LevelFilter::Debug;
    } else if args.contains(&"--info".to_string()) {
        log_level = LevelFilter::Info;
    }

    Builder::from_default_env()
        .filter_level(log_level)
        .init();
    let small_primes: Vec<u64> = (100..500000).filter(|&x| is_prime(x)).collect();
    let s_box = get_sbox(); 

    let filtered_args: Vec<String> = args.into_iter()
        .filter(|arg| !arg.starts_with("--"))
        .collect();

    if filtered_args.len() < 3 {
        println!("Usage: qosmic_512 [--debug|--info] (-f/-s) (file/string)");
		println!("Note: For multi-word string input, enclose the string in double quotes (e.g., qosmic_512 -s \"Hello, world!\")");
        process::exit(1);
    }

    let mode_arg_idx = 1;
    let input_arg_idx = 2;

    if filtered_args.len() <= input_arg_idx {
        println!("Error: Missing mode or input argument after flags.");
        process::exit(1);
    }

    let mode_arg = &filtered_args[mode_arg_idx];
    let input_arg = &filtered_args[input_arg_idx];

    if mode_arg != "-f" && mode_arg != "-s" {
        println!("Error: You must specify either -f or -s");
        process::exit(1);
    }

    let input_data_1: Vec<u8>;
    let fs_char_1: char;

    if mode_arg == "-f" {
        fs_char_1 = 'f';
        let mut file = fs::File::open(&input_arg).expect("Failed to open file");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("Failed to read file");
        input_data_1 = buffer;
    } else { // mode_arg == "-s"
        fs_char_1 = 's';
        input_data_1 = input_arg.as_bytes().to_vec();
    }

    let nonce_1 = derive_deterministic_nonce(&input_data_1);
    let hash_result_1 = qosmic512(input_data_1, fs_char_1, s_box, &small_primes, nonce_1);
    println!("qosmic_512 Hash: {}", hash_result_1);
}
