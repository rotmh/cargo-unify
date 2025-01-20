use super::*;

/// Path to the mock testing crate.
const TESTS_PATH: &str = "../_test/src";

#[test]
fn test_expand() {
    const EXPECTED: &str = "mod a {
mod c {
fn c() {}

} 

fn a() {}

} 
mod b {
mod d {
fn d() {}

} 

fn b() {}

} 

fn lib() {}
";

    let path = Path::new(TESTS_PATH).join("lib.rs");
    let content = read_path(&path).expect("path should be readable");
    let expanded = expand(&content, &path).expect("testing crate should be parseable");

    assert_eq!(expanded, EXPECTED.to_owned());
}
