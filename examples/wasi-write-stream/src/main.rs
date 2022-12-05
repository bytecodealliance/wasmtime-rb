use std::io::Write;

fn main() {
    for n in 1..100 {
        let s = n.to_string();
        std::io::stdout().write_all(s.repeat(100).as_bytes())
            .expect("failed to write to stdout");
    }
}
