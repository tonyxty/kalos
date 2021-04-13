#![feature(c_variadic)]

use std::cell::RefCell;

use lazy_static::lazy_static;

use kalos::run;
use rand::Rng;

thread_local! {
    static INPUT_BUF: RefCell<Option<Vec<i64>>> = RefCell::new(None);
    // Note that INPUT_BUF is fed to the program in reverse order
    static OUTPUT_BUF: RefCell<Option<Vec<i64>>> = RefCell::new(None);
}

unsafe extern "C" fn println(n: i64, mut args: ...) {
    for _ in 0..n {
        let val = args.arg::<i64>();
        OUTPUT_BUF.with(|output_buf| {
            let mut output_buf = output_buf.borrow_mut();
            let output_buf = output_buf.as_mut().unwrap();
            output_buf.push(val)
        });
    }
}

extern "C" fn read_int() -> i64 {
    INPUT_BUF.with(|input_buf| {
        let mut input_buf = input_buf.borrow_mut();
        let input_buf = input_buf.as_mut().unwrap();
        input_buf.pop().unwrap()
    })
}

lazy_static! {
    static ref TEST_RUNTIME: [(&'static str, usize); 2] = [
        ("println", println as usize),
        ("read_int", read_int as usize),
    ];
}

fn test_file(filename: &str, input: Vec<i64>, verify: impl FnOnce(&Vec<i64>) -> bool) {
    INPUT_BUF.with(|input_buf| input_buf.replace(Some(input)));
    OUTPUT_BUF.with(|output_buf| output_buf.replace(Some(Vec::new())));
    run(filename, &*TEST_RUNTIME);
    OUTPUT_BUF.with(|output_buf| {
        let output_buf = output_buf.borrow();
        let output_buf = output_buf.as_ref().unwrap();
        assert!(verify(output_buf));
    });
}

fn vec_equal<T: PartialEq>(lhs: &Vec<T>, rhs: &Vec<T>) -> bool {
    if lhs.len() != rhs.len() {
        return false;
    }
    for (x, y) in lhs.iter().zip(rhs) {
        if *x != *y {
            return false;
        }
    }
    true
}

#[test]
fn test_plus() {
    let mut rng = rand::thread_rng();
    let x: i64 = rng.gen();
    let y: i64 = rng.gen();
    test_file("examples/a+b.kls", vec![x, y], |v| v.len() == 1 && v[0] == x + y);
}

#[test]
fn test_chinese_remainder_theorem() {
    let mut rng = rand::thread_rng();
    let a = rng.gen::<i64>() % 3;
    let b = rng.gen::<i64>() % 5;
    let c = rng.gen::<i64>() % 7;
    test_file("examples/CRT.kls", vec![c, b, a],
              |v| v.len() == 1 && v[0] % 3 == a && v[0] % 5 == b && v[0] % 7 == c);
}

#[test]
fn test_hanoi() {
    test_file("examples/hanoi.kls", vec![3],
              |v| vec_equal(v, &vec![1, 3, 1, 2, 3, 2, 1, 3, 2, 1, 2, 3, 1, 3]));
}

#[test]
fn test_loop() {
    test_file("examples/loop.kls", Vec::new(),
              |v| vec_equal(v, &vec![625, 529, 441, 361, 289, 225, 169, 121, 81, 49, 25, 9, 1]));
}
