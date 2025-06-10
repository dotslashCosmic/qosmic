// src/main.rs
use qosmic_512::{get_sbox, qosmic512};
use qosmic_512::utils::derive_deterministic_nonce;
use qosmic_512::encode;
use std::io::BufRead;
use log::{LevelFilter, debug, info, error};
use env_logger::{Builder, Target};
use std::{env, fs::{self, File}, io::{self, Read, Write, BufReader, BufWriter}, process};

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let mut log_level = LevelFilter::Off;
    let mut debug_to_file = false;
    if args.contains(&"--version".to_string()) {
        println!("{} v{} by {}", env!("CARGO_PKG_DESCRIPTION"), env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_AUTHORS"));
        println!("{}", env!("CARGO_PKG_CUSTOM_COPYRIGHT"));
        process::exit(0);}
    if args.contains(&"--help".to_string()) {
        print_help();
        process::exit(0);}
    if args.contains(&"--debug".to_string()) {
        log_level = LevelFilter::Debug;
        debug_to_file = true;
    } else if args.contains(&"--info".to_string()) {
        log_level = LevelFilter::Info;}
    let mut logger_builder = Builder::from_default_env();
    if debug_to_file {
        let log_file_path = "qosmic_debug.txt";
        match File::create(log_file_path) {
            Ok(file) => {
                let writer = Box::new(BufWriter::new(file));
                logger_builder.target(Target::Pipe(writer));
                eprintln!("Debug logs (and higher) will be written to {}.", log_file_path);
            },
            Err(e) => {
                eprintln!("Warning: Could not create log file '{}': {}. Debug logs will go to stderr instead.", log_file_path, e);
                logger_builder.target(Target::Stderr);}}
    } else {
        logger_builder.target(Target::Stderr);}
    logger_builder
        .filter_level(log_level)
        .init();
    debug!("Application started with arguments: {:?}", args);
    if log_level == LevelFilter::Info {
        info!("Info logging enabled.");}
    let mut key: Option<Vec<u8>> = None;
    if let Some(pos) = args.iter().position(|r| r == "--key") {
        if pos + 1 < args.len() {
            key = Some(args[pos + 1].clone().as_bytes().to_vec());
            debug!("User-defined key detected. Length: {}", key.as_ref().unwrap().len());
            args.remove(pos + 1);
            args.remove(pos);
        } else {
            error!("Error: Missing user-defined key after --key flag.");
            print_usage_cli();
            process::exit(1);}}
    let mut output_format: Option<String> = None;
    if let Some(pos) = args.iter().position(|r| r == "-o") {
        if pos + 1 < args.len() {
            let format_arg = args[pos + 1].clone();
            match format_arg.as_str() {
                "b36" | "bin" | "hex" => {
                    output_format = Some(format_arg.clone());
                    debug!("Output format set to: {}", format_arg);},
                _ => {
                    error!("Error: Invalid output format '{}'. Use -o b36, -o bin, or -o hex.", format_arg);
                    print_usage_cli();
                    process::exit(1);}}
            args.remove(pos + 1);
            args.remove(pos);
        } else {
            error!("Error: Missing output format after -o flag. Use -o b36, -o bin, or -o hex.");
            print_usage_cli();
            process::exit(1);}}
    let mut batch_file_path: Option<String> = None;
    if let Some(pos) = args.iter().position(|r| r == "--batch-file") {
        if pos + 1 < args.len() {
            batch_file_path = Some(args[pos + 1].clone());
            debug!("Batch file mode enabled. Path: {}", batch_file_path.as_ref().unwrap());
            args.remove(pos + 1);
            args.remove(pos);
        } else {
            error!("Error: Missing file path after --batch-file flag.");
            print_usage_cli();
            process::exit(1);}}
    if args.contains(&"--interactive".to_string()) {
        if batch_file_path.is_some() {
            error!("Error: Cannot use --interactive and --batch-file together.");
            print_usage_cli();
            process::exit(1);}
        info!("Running in interactive mode.");
        run_interactive_mode(key, output_format);
    } else if let Some(path) = batch_file_path {
        info!("Running in batch file mode.");
        run_batch_mode(path, key, output_format);
    } else {
        info!("Running in CLI mode.");
        run_cli_mode(args, key, output_format);}
    debug!("Application finished.");}

fn print_help() {
    println!("{}\n", env!("CARGO_PKG_DESCRIPTION"));
    println!("Usage: qosmic_512 [OPTIONS] <INPUT_MODE> <INPUT_VALUE>\n");
    println!("Calculate the qosmic512 hash of a string or file.\n");
    println!("Input Modes:");
    println!("  -f <file>      Specify a file path as input.");
    println!("  -s <string>    Specify a string literal as input.\n");
    println!("Options:");
    println!("  --help         Display this help message and exit.");
    println!("  --version      Display version information and exit.");
    println!("  --debug        Enable debug logging (writes to qosmic_debug.txt or stderr).");
    println!("  --info         Enable info logging (writes to stderr).");
    println!("  -o <format>    Specify output format: 'b36' (Base36), 'b58' (Base58), 'b64' (Base64), 'bin' (Binary), 'hex' (Hex, default)");
    println!("  --key <key>    Specify a user-defined key for hashing. For multi-word input, enclose in double quotes.");
    println!("  --interactive  Run in interactive mode, processing input line by line from stdin.");
    println!("  --batch-file <file> Process lines from a file as input, outputting hashes one per line for max performance.\n");
    println!("Examples:");
    println!("  qosmic_512 -s \"Hello World\" -o b36");
    println!("  qosmic_512 -f my_document.txt --key \"mysecretkey\" --debug");
    println!("  qosmic_512 --interactive -o bin");
    println!("  qosmic_512 --batch-file nonce_list.txt -o hex\n");
    println!("Note: For multi-word string/key input, enclose the value in double quotes.");}

fn print_usage_cli() {
    println!("Usage: qosmic_512 [--debug|--info] (-f <file> | -s <string> | --interactive | --batch-file <file>) [-o b36|bin|hex] [--key <key>] [--version|--help]");
    println!("For detailed help, run: qosmic_512 --help");}

fn run_cli_mode(args: Vec<String>, pre_set_key: Option<Vec<u8>>, output_format: Option<String>) {
    debug!("run_cli_mode: Arguments (filtered): {:?}", args);
    let s_box = get_sbox();
    let key_bytes: Option<&[u8]> = pre_set_key.as_deref();
    let filtered_args: Vec<String> = args.into_iter()
        .filter(|arg| !arg.starts_with("--") && !arg.starts_with("-o"))
        .collect();
    if filtered_args.len() < 3 {
        error!("Error: Missing arguments for CLI mode.");
        print_usage_cli();
        process::exit(1);}
    let mode_arg_idx = 1;
    let input_arg_idx = 2;
    let mode_arg = &filtered_args[mode_arg_idx];
    let input_arg = &filtered_args[input_arg_idx];
    if mode_arg != "-f" && mode_arg != "-s" {
        error!("Error: You must specify either -f (file input) or -s (string input).");
        print_usage_cli();
        process::exit(1);}
    let (input_data, _fs_char) = if mode_arg == "-f" {
        info!("Reading input from file: {}", input_arg);
        match fs::File::open(input_arg) {
            Ok(mut file) => {
                let mut buffer = Vec::new();
                if let Err(e) = file.read_to_end(&mut buffer) {
                    error!("Failed to read file: {}", e);
                    process::exit(1);
                }
                debug!("Successfully read {} bytes from file.", buffer.len());
                (buffer, 'f')
            },
            Err(e) => {
                error!("Failed to open file '{}': {}", input_arg, e);
                process::exit(1);}}
    } else {
        info!("Using string input: '{}'", input_arg);
        debug!("String input as bytes: {:?}", input_arg.as_bytes());
        (input_arg.as_bytes().to_vec(), 's')};
    let nonce = derive_deterministic_nonce(&input_data);
    debug!("Derived deterministic nonce: {}", nonce);
    info!("Calculating qosmic512 hash...");
    let hash_result = qosmic512(input_data.to_vec(), 's', s_box, nonce, key_bytes);
    info!("Hash calculation complete.");
    let final_output = if let Some(format) = output_format {
        match format.as_str() {
            "b36" => {
                debug!("Encoding hash to Base36.");
                encode::to_base36(&hash_result)},
            "b58" => {
                debug!("Encoding hash to Base58.");
                encode::to_base58(&hash_result)},
            "b64" => {
                debug!("Encoding hash to Base64.");
                encode::to_base64(&hash_result)},
            "bin" => {
                debug!("Encoding hash to Binary.");
                encode::to_binary(&hash_result)},
            "hex" => {
                debug!("Outputting hash in Hexadecimal.");
                hash_result},
            _ => hash_result,}
    } else {
        debug!("Outputting hash in default Hexadecimal format.");
        hash_result};
    println!("{}", final_output);
    debug!("Output printed to stdout.");}

fn run_interactive_mode(key: Option<Vec<u8>>, initial_output_format: Option<String>) {
    info!("Interactive mode active. Type input and press Enter. Press Ctrl+D (Unix) or Ctrl+Z then Enter (Windows) to exit.");
    let s_box = get_sbox();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut buffer = String::new();
    let key_slice = key.as_deref();
    let output_format = initial_output_format;
    loop {
        debug!("Waiting for input in interactive mode...");
        if stdout.flush().is_err() {
            error!("Error flushing stdout in interactive mode.");
            break;}
        buffer.clear();
        match stdin.read_line(&mut buffer) {
            Ok(0) => {
                info!("End of input received (EOF). Exiting interactive mode.");
                break;},
            Ok(n) => {
                let input = buffer.trim();
                debug!("Received {} bytes input: '{}'", n, input);
                if input.is_empty() {
                    debug!("Input is empty, skipping.");
                    continue;}
                let input_bytes = input.as_bytes();
                debug!("Input bytes for hashing: {:?}", input_bytes);
                let nonce = derive_deterministic_nonce(input_bytes);
                debug!("Derived nonce for interactive input: {}", nonce);
                let hash_result = qosmic512(input_bytes.to_vec(), 's', s_box, nonce, key_slice);
                debug!("Hash result (hex): {}", hash_result);
                let final_output = if let Some(format) = &output_format {
                    match format.as_str() {
                        "b36" => {
                            debug!("Encoding hash to Base36.");
                            encode::to_base36(&hash_result)},
                        "b58" => {
                            debug!("Encoding hash to Base58.");
                            encode::to_base58(&hash_result)},
                        "b64" => {
                            debug!("Encoding hash to Base64.");
                            encode::to_base64(&hash_result)},
                        "bin" => {
                            debug!("Encoding hash to Binary.");
                            encode::to_binary(&hash_result)},
                        "hex" => {
                            debug!("Outputting interactive hash in Hexadecimal.");
                            hash_result.clone()},
                        _ => hash_result.clone(),}
                } else {
                    debug!("Outputting interactive hash in default Hexadecimal format.");
                    hash_result};
                if writeln!(stdout, "{}", final_output).is_err() {
                    error!("Error writing to stdout in interactive mode.");
                    break;}
                if stdout.flush().is_err() {
                    error!("Error flushing stdout after write in interactive mode.");
                    break;}
                debug!("Output for interactive input printed.");},
            Err(error) => {
                error!("Error reading from stdin in interactive mode: {}", error);
                break;}}}}

fn run_batch_mode(file_path: String, key: Option<Vec<u8>>, output_format: Option<String>) {
    info!("Batch mode active. Processing file: {}", file_path);
    let s_box = get_sbox();
    let key_slice = key.as_deref();
    let file = match File::open(&file_path) {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to open batch file '{}': {}", file_path, e);
            process::exit(1);}};
    let reader = BufReader::new(file);
    let mut stdout_buf = BufWriter::new(io::stdout());
    let mut line_count = 0;
    for line in reader.lines() {
        line_count += 1;
        let input = match line {
            Ok(l) => l.trim().to_string(),
            Err(e) => {
                error!("Error reading line {} from batch file: {}", line_count, e);
                continue;}};
        if input.is_empty() {
            debug!("Line {} is empty, skipping.", line_count);
            continue;}
        let input_bytes = input.as_bytes();
        debug!("Processing line {}: '{}' (bytes: {:?})", line_count, input, input_bytes);
        let nonce = derive_deterministic_nonce(input_bytes);
        debug!("Derived nonce for line {}: {}", line_count, nonce);
        let hash_result = qosmic512(input_bytes.to_vec(), 's', s_box, nonce, key_slice);
        debug!("Hash result for line {} (hex): {}", line_count, hash_result);
        let final_output = if let Some(format) = &output_format {
            match format.as_str() {
                "b36" => {
                    debug!("Encoding hash for line {} to Base36.", line_count);
                    encode::to_base36(&hash_result)},
                "b58" => {
                    debug!("Encoding hash for line {} to Base58.", line_count);
                    encode::to_base58(&hash_result)},
                "b64" => {
                    debug!("Encoding hash for line {} to Base64.", line_count);
                    encode::to_base646(&hash_result)},
                "bin" => {
                    debug!("Encoding hash for line {} to Binary.", line_count);
                    encode::to_binary(&hash_result)},
                "hex" => {
                    debug!("Outputting hash for line {} in Hexadecimal.", line_count);
                    hash_result.clone()},
                _ => hash_result.clone(),}
        } else {
            debug!("Outputting hash for line {} in default Hexadecimal format.", line_count);
            hash_result};
        if writeln!(stdout_buf, "{}", final_output).is_err() {
            error!("Error writing output for line {} to stdout in batch mode.", line_count);
            break;}}
    if stdout_buf.flush().is_err() {
        error!("Error flushing stdout at the end of batch mode.");}
    info!("Finished processing {} lines in batch mode.", line_count);}
