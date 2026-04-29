#[test]
fn effect_macro_compile_passes() {
  let cases = trybuild::TestCases::new();
  cases.pass("tests/ui/effect_macro_tail_bind.rs");
  cases.pass("tests/ui/effect_macro_nested_block_bind.rs");
  cases.pass("tests/ui/effect_macro_async_block_bind.rs");
  cases.pass("tests/ui/effect_macro_turbofish_bind.rs");
  cases.pass("tests/ui/effect_macro_bind_precedence.rs");
}

#[test]
fn effect_macro_compile_failures() {
  let cases = trybuild::TestCases::new();
  cases.compile_fail("tests/ui/effect_macro_invalid_bind_syntax.rs");
}
