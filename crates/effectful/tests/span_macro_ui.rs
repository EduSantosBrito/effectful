#[test]
fn span_macro_compile_failures() {
  let cases = trybuild::TestCases::new();
  cases.compile_fail("tests/ui/span_non_debug_arg.rs");
  cases.compile_fail("tests/ui/span_non_effect_return.rs");
  cases.pass("tests/ui/effect_test_no_tokio_import.rs");
  cases.pass("tests/ui/span_non_debug_skipped.rs");
}
