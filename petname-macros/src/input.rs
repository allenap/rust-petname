use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Token};

/// Input to the macros in this crate.
///
/// An optional unnamed directory prefix may be followed by named arguments for
/// each word list. Individual paths are resolved relative to the directory if
/// one is given, or used directly otherwise.
pub struct PetnamesInput {
    pub dir: Option<LitStr>,
    pub adjectives: Option<LitStr>,
    pub adverbs: Option<LitStr>,
    pub nouns: Option<LitStr>,
}

impl Parse for PetnamesInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut dir = None;
        let mut adjectives = None;
        let mut adverbs = None;
        let mut nouns = None;

        // Optional unnamed directory argument.
        if input.peek(LitStr) {
            dir = Some(input.parse::<LitStr>()?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        // Named arguments in any order.
        while !input.is_empty() {
            let name: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitStr = input.parse()?;
            match name.to_string().as_str() {
                "adjectives" if adjectives.is_none() => adjectives = Some(value),
                "adverbs" if adverbs.is_none() => adverbs = Some(value),
                "nouns" if nouns.is_none() => nouns = Some(value),
                "adjectives" | "adverbs" | "nouns" => {
                    return Err(syn::Error::new(name.span(), format!("duplicate argument `{name}`")));
                }
                other => {
                    return Err(syn::Error::new(
                        name.span(),
                        format!(
                            "unexpected argument `{other}`, expected `adjectives`, `adverbs`, or `nouns`"
                        ),
                    ));
                }
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(PetnamesInput { dir, adjectives, adverbs, nouns })
    }
}
