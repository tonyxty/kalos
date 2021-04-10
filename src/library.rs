use std::io::stdin;

pub unsafe extern "C" fn println(n: i64, mut args: ...) -> i64 {
    let mut values = Vec::new();
    for _ in 0..n {
        values.push(args.arg::<i64>().to_string());
    }
    println!("{}", values.join(" "));
    n
}

pub extern "C" fn read_int() -> i64 {
    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();
    buf.trim().parse().unwrap()
}
