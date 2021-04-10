use std::io::stdin;

pub extern "C" fn println(x: i64) {
    println!("{}", x);
}

pub extern "C" fn read_int() -> i64 {
    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();
    buf.trim().parse().unwrap()
}
