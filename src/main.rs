use rust_life;
use std::env;

fn main() {
    if let Err(e) = rust_life::run(env::args()) {
        panic!("Application error: {}", e);
    };
}
