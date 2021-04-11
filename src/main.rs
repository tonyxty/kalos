use kalos::run;

fn main() {
    let filename = std::env::args().nth(1).expect("some arg thing failed");
    run(&filename);
}
