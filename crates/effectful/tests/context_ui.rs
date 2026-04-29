#[test]
fn context_compile_passes() {
  let cases = trybuild::TestCases::new();
  cases.pass("tests/ui/context_get_after_head.rs");
  cases.pass("tests/ui/context_service_context_after_tail_lookup.rs");
}
