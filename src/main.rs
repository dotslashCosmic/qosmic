// src/main.rs
use qosmic_512::{qosmic512, get_sbox, is_prime};
use std::{env, fs, io::{self, Read, Write}, process};
use qosmic_512::utils::derive_deterministic_nonce;
use log::LevelFilter;
use env_logger::Builder;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut log_level = LevelFilter::Off;
    if args.contains(&"--debug".to_string()) {
        log_level = LevelFilter::Debug;
    } else if args.contains(&"--info".to_string()) {
        log_level = LevelFilter::Info;}
    Builder::from_default_env()
        .filter_level(log_level)
        .init();
    if args.contains(&"--interactive".to_string()) {
        run_interactive_mode();
    } else {
        run_cli_mode(args);}}

fn run_cli_mode(mut args: Vec<String>) {
    let small_primes: Vec<u64> = (100..500000).filter(|&x| is_prime(x)).collect();
    let s_box = get_sbox();
    let mut key: Option<String> = None;
    if let Some(pos) = args.iter().position(|r| r == "--key") {
        if pos + 1 < args.len() {
            key = Some(args[pos + 1].clone());
            args.remove(pos + 1);
            args.remove(pos);
        } else {
            println!("Error: Missing user-defined key after --key flag.");
            process::exit(1);}}
    let filtered_args: Vec<String> = args.into_iter()
        .filter(|arg| !arg.starts_with("--"))
        .collect();
    if filtered_args.len() < 3 {
        println!("Usage: qosmic_512 [--debug|--info] (-f <file> | -s <string>) [--key <key>]");
		println!("Note: For multi-word string/key input, enclose the value in double quotes.");
        process::exit(1);}
    let mode_arg_idx = 1;
    let input_arg_idx = 2;
    if filtered_args.len() <= input_arg_idx {
        println!("Error: Missing mode or input argument after flags.");
        process::exit(1);}
    let mode_arg = &filtered_args[mode_arg_idx];
    let input_arg = &filtered_args[input_arg_idx];
    if mode_arg != "-f" && mode_arg != "-s" {
        println!("Error: You must specify either -f or -s");
        process::exit(1);}
    let (input_data, fs_char) = if mode_arg == "-f" {
        let mut file = fs::File::open(&input_arg).expect("Failed to open file");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("Failed to read file");
        (buffer, 'f')
    } else {
        (input_arg.as_bytes().to_vec(), 's')};
    let key_bytes = key.as_deref().map(|k| k.as_bytes());
    let nonce = derive_deterministic_nonce(&input_data);
    let hash_result = qosmic512(input_data, fs_char, s_box, &small_primes, nonce, key_bytes);
    println!("qosmic_512 Hash: {}", hash_result);}
fn run_interactive_mode() {
    let small_primes: Vec<u64> = (100..500000).filter(|&x| is_prime(x)).collect();
    let s_box = get_sbox();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut buffer = String::new();
    let mut key: Option<Vec<u8>> = None;
    loop {
        buffer.clear();
        match stdin.read_line(&mut buffer) {
            Ok(0) => {
                break;},
            Ok(_) => {
                let input = buffer.trim();
                if input.eq_ignore_ascii_case("EXIT") || input.eq_ignore_ascii_case("QUIT") {
                    break;}
                if input.to_uppercase().starts_with("KEY ") {
                    let key_str = &input[4..];
                    key = Some(key_str.as_bytes().to_vec());
                    writeln!(stdout, "Key set for subsequent hashes.").unwrap();
                    continue;}
                let input_bytes = input.as_bytes().to_vec();
                if input_bytes.is_empty() {
                    writeln!(stdout, "Error: Empty input received.").unwrap();
                    continue;}
                let nonce = derive_deterministic_nonce(&input_bytes);
                let key_slice = key.as_deref();
                let hash_result = qosmic512(input_bytes, 's', s_box, &small_primes, nonce, key_slice);
                writeln!(stdout, "{}", hash_result).unwrap();
                stdout.flush().unwrap();},
            Err(error) => {
                eprintln!("Error reading from stdin in interactive mode: {}", error);
                break;}}}}
