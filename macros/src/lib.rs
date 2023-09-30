
use proc_macro::TokenStream;
use quote::quote;
use syn::{LitStr, parse_macro_input};

#[proc_macro]
pub fn gene(input: TokenStream) -> TokenStream {
    let name = parse_macro_input!(input as LitStr);
    let mut sequence: Vec<u8> = name.value().into_bytes();
    for c in sequence.iter_mut() {
        const SEQ: [u8; 4] = [b'a', b'c', b'g', b't'];
        *c = SEQ[usize::from(*c) % 4];
    }
    let sequence = String::from_utf8(sequence).unwrap();
    quote!{
        crate::organism::gene::Gene {
            name: #name,
            sequence: #sequence,
        }
    }.into()
}

