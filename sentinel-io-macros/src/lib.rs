use proc_macro::{Delimiter, TokenStream, TokenTree};

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let body = item
        .into_iter()
        .filter_map(|tt| match tt {
            TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => Some(g.stream()),
            _ => None,
        })
        .last()
        .expect("expected function body");

    format!(
        "fn main() {{ sentinel_io::block_on(async {{ {} }}); }}",
        body
    )
    .parse()
    .unwrap()
}
