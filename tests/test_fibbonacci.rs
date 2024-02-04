#[test]
fn test_fibbonacci() {
    let script_path = "./tests/data/fibbonacci.rlox";
    rlox::lox::Lox::new().run_file(script_path).unwrap();
}
