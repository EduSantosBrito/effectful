#[test]
fn test_bind_star_tokenization() {
  use proc_macro2::TokenStream;
  use quote::quote;

  let tokens: TokenStream = quote! {
      bind* succeed(10)
  };

  println!("Tokens: {}", tokens);
  for tt in tokens {
    println!("Token kind: {:?}", tt);
  }
}
