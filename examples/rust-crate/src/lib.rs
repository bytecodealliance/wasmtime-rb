pub fn create_an_engine() -> wasmtime_rb::Engine {
    wasmtime_rb::Engine::new(&[]).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let _cleanup = unsafe { magnus::embed::init() };
        let engine = create_an_engine();

        assert!(engine.get().precompile_module(b"(module)").is_ok())
    }
}
