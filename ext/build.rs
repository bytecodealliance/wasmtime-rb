use std::error::Error;

// Propogate linking from rb-sys for usage in the wasmtime-rb Rust crate
fn main() -> Result<(), Box<dyn Error>> {
    let _ = rb_sys_env::activate()?;

    Ok(())
}
