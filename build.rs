// build.rs
use std::{env, fs, path::Path};

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        winres::WindowsResource::new()
            .set_resource_file("app.rc")
            .compile()
            .unwrap();}
    println!("cargo:rerun-if-changed=Cargo.toml");
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set");
    let cargo_toml_path = Path::new(&manifest_dir).join("Cargo.toml");
    let cargo_toml_content = fs::read_to_string(&cargo_toml_path)
        .expect("Failed to read Cargo.toml");
    let mut found_copyright = false;
    for line in cargo_toml_content.lines() {
        let trimmed_line = line.trim();
        if trimmed_line.starts_with("copyright = \"") && trimmed_line.ends_with("\"") {
            if let Some(start_quote) = trimmed_line.find('"') {
                if let Some(end_quote) = trimmed_line.rfind('"') {
                    if end_quote > start_quote {
                        let copyright_value = &trimmed_line[start_quote + 1..end_quote];
                        println!("cargo:rustc-env=CARGO_PKG_CUSTOM_COPYRIGHT={}", copyright_value);
                        eprintln!("Build script: Set CARGO_PKG_CUSTOM_COPYRIGHT to '{}'", copyright_value);
                        found_copyright = true;
                        break;}}}}}
    if !found_copyright {
        eprintln!("Build script: 'copyright' field not found in Cargo.toml or invalid format.");
        println!("cargo:rustc-env=CARGO_PKG_CUSTOM_COPYRIGHT=");}}
