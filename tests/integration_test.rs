#![feature(c_variadic)]

use lazy_static::lazy_static;

use kalos::run;

// ** Important **  Due to the use of globals, the tests must be run with --test-threads=1

static mut OUTPUT_BUF: Option<Vec<i64>> = None;

unsafe extern "C" fn println(n: i64, mut args: ...) {
    for _ in 0..n {
        OUTPUT_BUF.as_mut().unwrap().push(args.arg::<i64>());
    }
}

static mut INPUT_BUF: Option<Vec<i64>> = None;

extern "C" fn read_int() -> i64 {
    unsafe { INPUT_BUF.as_mut() }.unwrap().pop().unwrap()
}

lazy_static! {
    static ref TEST_RUNTIME: [(&'static str, usize); 2] = [
        ("println", println as usize),
        ("read_int", read_int as usize),
    ];
}

fn test_file(filename: &str, input: Vec<i64>, output: Vec<i64>) {
    unsafe {
        INPUT_BUF = Some(input);
        OUTPUT_BUF = Some(Vec::new());
    }
    run(filename, &*TEST_RUNTIME);
    let output_buf = unsafe { OUTPUT_BUF.as_ref() }.unwrap();
    assert_eq!(output_buf.len(), output.len());
    for (x, y) in output_buf.iter().zip(&output) {
        assert_eq!(*x, *y);
    }
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
