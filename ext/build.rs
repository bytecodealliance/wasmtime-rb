use std::error::Error;

// Propogate linking from rb-sys for usage in the wasmtime-rb Rust crate
fn main() -> Result<(), Box<dyn Error>> {
    let _ = rb_sys_env::activate()?;
    let out_dir = std::env::var("OUT_DIR")?;
    let error_rb = std::path::Path::new(&out_dir).join("error.rb");
    std::fs::copy("../lib/wasmtime/error.rb", error_rb)?;

    Ok(())
}
