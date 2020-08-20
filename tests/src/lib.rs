#[test]
fn trybuild() {
    let directory = std::path::Path::new(".");

    let testcases = trybuild::TestCases::new();
    testcases.compile_fail(directory.join("compile_fail/*.rs"));
    testcases.pass(directory.join("pass/*.rs"));
}
