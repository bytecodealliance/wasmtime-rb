use miniserde::{json, Serialize};
use std::io::{Write, Read};

#[derive(Serialize)]
struct Wasi {
    args: Vec<String>,
    env: Vec<(String, String)>,
    pwd: String,
    stdin: String,
}

#[derive(Serialize)]
struct Log<'a> {
    name: &'static str,
    wasi: &'a Wasi,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let env: Vec<(String, String)> = std::env::vars().collect();
    let pwd : String = std::env::current_dir()
        .expect("current working directory")
        .to_string_lossy()
        .into();

    let mut stdin = String::new();
    std::io::stdin().read_to_string(&mut stdin)
        .expect("failed to read stdin");

    let wasi = Wasi {args, env, pwd, stdin };
    let stdout = Log { name: "stdout", wasi: &wasi };
    let stderr = Log { name: "stderr", wasi: &wasi };

    std::io::stdout().write_all(json::to_string(&stdout).as_bytes())
        .expect("failed to write to stdout");
    std::io::stderr().write_all(json::to_string(&stderr).as_bytes())
        .expect("failed to write to stderr");
}
