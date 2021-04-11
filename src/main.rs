use kalos::run;

fn main() -> anyhow::Result<()> {
    let filename = std::env::args().nth(1).expect("some arg thing failed");
    run(&filename)
}
