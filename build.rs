// build.rs
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        winres::WindowsResource::new()
            .set_resource_file("app.rc")
            .compile()
            .unwrap();}}