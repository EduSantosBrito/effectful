#[test]
fn context_compile_passes() {
  let cases = trybuild::TestCases::new();
  cases.pass("tests/ui/context_get_after_head.rs");
}
