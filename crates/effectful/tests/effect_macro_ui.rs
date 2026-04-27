#[test]
fn effect_macro_compile_passes() {
  let cases = trybuild::TestCases::new();
  cases.pass("tests/ui/effect_macro_tail_bind.rs");
}
