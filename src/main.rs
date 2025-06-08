// src/main.rs
use qosmic_512::{get_sbox, is_prime, qosmic512};
use qosmic_512::utils::derive_deterministic_nonce;
use log::LevelFilter;
use env_logger::Builder;
use std::{env, fs, io::{self, Read, Write}, process};

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let mut log_level = LevelFilter::Off;
    if args.contains(&"--debug".to_string()) {
        log_level = LevelFilter::Debug;
    } else if args.contains(&"--info".to_string()) {
        log_level = LevelFilter::Info;}
    Builder::from_default_env()
        .filter_level(log_level)
        .init();
    let mut key: Option<Vec<u8>> = None;
    if let Some(pos) = args.iter().position(|r| r == "--key") {
        if pos + 1 < args.len() {
            key = Some(args[pos + 1].clone().as_bytes().to_vec());
            args.remove(pos + 1);
            args.remove(pos);
        } else {
            eprintln!("Error: Missing user-defined key after --key flag.");
            process::exit(1);}}
    if args.contains(&"--interactive".to_string()) {
        run_interactive_mode(key);
    } else {
        run_cli_mode(args, key);}}

fn run_cli_mode(mut args: Vec<String>, pre_set_key: Option<Vec<u8>>) {
    let s_box = get_sbox();
    let key_bytes: Option<&[u8]> = pre_set_key.as_deref();
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
        eprintln!("Error: Missing mode or input argument after flags.");
        process::exit(1);}
    let mode_arg = &filtered_args[mode_arg_idx];
    let input_arg = &filtered_args[input_arg_idx];
    if mode_arg != "-f" && mode_arg != "-s" {
        eprintln!("Error: You must specify either -f or -s");
        process::exit(1);}
    let (input_data, fs_char) = if mode_arg == "-f" {
        match fs::File::open(input_arg) {
            Ok(mut file) => {
                let mut buffer = Vec::new();
                if let Err(e) = file.read_to_end(&mut buffer) {
                    eprintln!("Failed to read file: {}", e);
                    process::exit(1);}
                (buffer, 'f')},
            Err(e) => {
                eprintln!("Failed to open file '{}': {}", input_arg, e);
                process::exit(1);}}
    } else {
        (input_arg.as_bytes().to_vec(), 's')};
    let nonce = derive_deterministic_nonce(&input_data);
    let hash_result = qosmic512(input_data, 's', s_box, nonce, key_bytes);
    println!("{}", hash_result);}

fn run_interactive_mode(key: Option<Vec<u8>>) {
    let s_box = get_sbox();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut buffer = String::new();
    let key_slice = key.as_deref();
    loop {
        buffer.clear();
        match stdin.read_line(&mut buffer) {
            Ok(0) => {
                break;}
            Ok(_) => {
                let input = buffer.trim();
                if input.is_empty() {
                    continue;}
                let input_bytes = input.as_bytes().to_vec();
                let nonce = derive_deterministic_nonce(&input_bytes);
                let hash_result = qosmic512(input_bytes, 's', s_box, nonce, key_slice);
                if writeln!(stdout, "{}", hash_result).is_err() {
                    break;}
                if stdout.flush().is_err() {
                    break;}}
            Err(error) => {
                eprintln!("Error reading from stdin in interactive mode: {}", error);
                break;}}}}
