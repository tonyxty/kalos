use kalos::{run, DEFAULT_RUNTIME};

fn main() {
    let filename = std::env::args().nth(1).expect("some arg thing failed");
    run(&filename, &*DEFAULT_RUNTIME);
}
