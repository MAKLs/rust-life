use rust_life;

fn main() {
    if let Err(e) = rust_life::run() {
        eprintln!("Application error: {}", e);
    };
}
