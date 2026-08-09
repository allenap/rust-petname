use proc_macro::TokenStream;

use crate::{
    lang::{embed, resolve, WordList},
    text::read_and_process,
};

/// See [`english!`][`crate::english!`] for documentation.
pub fn expand(input: TokenStream, macro_name: &str) -> TokenStream {
    let paths = resolve(input, macro_name);
    embed(
        "english",
        &paths,
        WordList::strings(read_and_process(&paths.adjectives)),
        WordList::strings(read_and_process(&paths.adverbs)),
        WordList::strings(read_and_process(&paths.nouns)),
    )
}
