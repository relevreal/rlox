#[test]
fn test_blocks() {
    let script_path = "./tests/data/blocks.rlox";
    rlox::lox::Lox::new().run_file(script_path).unwrap();
}
