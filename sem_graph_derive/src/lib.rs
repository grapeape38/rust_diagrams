extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(SendUniforms)]
pub fn derive_send_uniforms(input: proc_macro::TokenStream) -> 
proc_macro::TokenStream
{
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let field_tokens = fields_send_uniforms(&input.data);
    let expanded = quote! {
        impl SendUniforms for #name {
            fn send_uniforms(&self, prog_id: GLuint) -> Result<(), String> {
                #field_tokens
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

fn fields_send_uniforms(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let send_fields = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        let name_str = name.clone().unwrap().to_string();
                        quote_spanned! {
                            f.span() => self.#name.send_uniform(prog_id, #name_str)?;
                        }
                    });
                    quote! { #( #send_fields )* Ok(()) }
                }
                _ => unimplemented!()
            }
        }
        _ => unimplemented!()
    }
}




