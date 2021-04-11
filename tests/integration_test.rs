#![feature(c_variadic)]

use std::cell::RefCell;

use lazy_static::lazy_static;

use kalos::run;

thread_local! {
    static INPUT_BUF: RefCell<Option<Vec<i64>>> = RefCell::new(None);
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

fn test_file(filename: &str, input: Vec<i64>, output: Vec<i64>) {
    INPUT_BUF.with(|input_buf| input_buf.replace(Some(input)));
    OUTPUT_BUF.with(|output_buf| output_buf.replace(Some(Vec::new())));
    run(filename, &*TEST_RUNTIME);
    OUTPUT_BUF.with(|output_buf| {
        let output_buf = output_buf.borrow();
        let output_buf = output_buf.as_ref().unwrap();
        assert_eq!(output_buf.len(), output.len());
        for (x, y) in output_buf.iter().zip(&output) {
            assert_eq!(*x, *y);
        }
    });
}

#[test]
fn test_plus() {
    test_file("examples/a+b.kls", vec![3, 4], vec![7]);
}

#[test]
fn test_chinese_remainder_theorem() {
    test_file("examples/CRT.kls", vec![2, 3, 2], vec![23]);
}

#[test]
fn test_hanoi() {
    test_file("examples/hanoi.kls", vec![3], vec![1, 3, 1, 2, 3, 2, 1, 3, 2, 1, 2, 3, 1, 3]);
}

#[test]
fn test_loop() {
    test_file("examples/loop.kls", Vec::new(),
              vec![625, 529, 441, 361, 289, 225, 169, 121, 81, 49, 25, 9, 1]);
}
