#[allow(dead_code)]
pub fn runner(tests: &[&dyn Fn()]) {
    for test in tests {
        test();
    }
}
