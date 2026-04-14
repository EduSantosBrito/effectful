//! Ex 085 — `decode_unknown` accepts tree-shaped `Unknown`.
use effect_rs::EffectData;
use effect_rs::schema::{Unknown, string};

#[derive(Clone, Debug, EffectData)]
struct T;

fn main() {
  let s = string::<T>();
  assert!(s.decode_unknown(&Unknown::Null).is_err());
  assert_eq!(
    s.decode_unknown(&Unknown::String("hi".to_owned())),
    Ok("hi".to_owned())
  );
  println!("085_unknown_decode ok");
}
