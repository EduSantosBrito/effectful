#[test]
fn test_yield_tokenization() {
    use proc_macro2::TokenStream;
    use quote::quote;
    
    let tokens: TokenStream = quote! {
        yield* succeed(10)
    };
    
    println!("Tokens: {}", tokens);
    for tt in tokens {
        println!("Token kind: {:?}", tt);
    }
}
