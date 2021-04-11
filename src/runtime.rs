use std::io::stdin;

unsafe extern "C" fn println(n: i64, mut args: ...) {
    let mut values = Vec::new();
    for _ in 0..n {
        values.push(args.arg::<i64>().to_string());
    }
    println as usize;
    println!("{}", values.join(" "));
}

extern "C" fn read_int() -> i64 {
    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();
    buf.trim().parse().unwrap()
}

lazy_static! {
    pub static ref DEFAULT_RUNTIME: [(&'static str, usize); 2] = [
        ("println", println as usize),
        ("read_int", read_int as usize),
    ];
}
