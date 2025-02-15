use anyhow::Result;
use darling::FromDeriveInput;
use proc_macro2::{self, TokenStream};
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Field, Fields, GenericParam, Generics, Ident, Lifetime, Variant,
    __private::TokenStream2, punctuated::Punctuated, token::Comma,
};

#[derive(FromDeriveInput, Default, Clone)]
#[darling(default, attributes(osec))]
struct Opts {
    vector_length: u32,
    t: Option<String>,
}

pub fn arbitrary(input: TokenStream) -> Result<TokenStream> {
    let input = syn::parse2::<DeriveInput>(input)?;
    let opts = Opts::from_derive_input(&input).expect("Wrong options");
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = input;
    let result = match data {
        Data::Struct(data) => {
            let fields = data.fields;
            get_fields_tokenstream(fields)
        }
        Data::Enum(data) => {
            let variants = data.variants;
            let init_fields = get_variants_tokenstream(variants);

            quote! {
                use #ident::*;

                #init_fields
            }
        }
        _ => quote! {},
    };
    let implementation = quote! {
        fn any() -> Self {
            #result
        }
    };
    let lifetimes = parse_generics(generics);
    let output = get_impl_tokenstream(ident, implementation, lifetimes, &opts);
    Ok(output)
}

fn get_fields_tokenstream(fields: Fields) -> TokenStream2 {
    let result = match fields {
        Fields::Named(fields) => {
            let named_fields = fields.named;
            let mut init_fields = quote! {};
            for field in named_fields {
                let Field { ident, .. } = field;
                let field_ident = ident.unwrap();
                if field_ident.to_string().starts_with('_') {
                    continue;
                }
                init_fields.extend(quote! { #field_ident: kani::any(), });
            }
            init_fields
        }
        _ => quote! {},
    };
    quote! {
        Self{#result}
    }
}

fn get_variants_tokenstream(variants: Punctuated<Variant, Comma>) -> TokenStream2 {
    let mut init_fields = quote! {};
    for (index, variant) in (0_u8..).zip(variants.iter()) {
        init_fields.extend({
            quote! {
                #index => #variant,
            }
        });
    }
    let variant = variants.last();
    init_fields.extend({
        quote! {
            _ => #variant
        }
    });
    quote! {
        match kani::any::<u8>() {
            #init_fields
        }
    }
}

fn get_impl_tokenstream(
    ident: Ident,
    implementation: TokenStream2,
    lifetimes: Vec<Lifetime>,
    opts: &Opts,
) -> TokenStream2 {
    match &opts.t {
        Some(s) => {
            let mut results = vec![];
            for generic in s.split(',') {
                let generic_ident = format_ident!("{}", generic.trim());
                if lifetimes.is_empty() {
                    results.extend(quote! {
                        impl kani::Arbitrary for #ident<#generic_ident> {
                            #implementation
                        }
                    });
                } else {
                    results.extend(
                        quote! {
                            impl<#(#lifetimes),*, #generic_ident: kani::Arbitrary> kani::Arbitrary for #ident<#(#lifetimes),*, #generic_ident> {
                                #implementation
                            }
                        }
                    );
                }
            }
            quote! { #(#results)* }
        }
        None => {
            if lifetimes.is_empty() {
                quote! {
                    impl kani::Arbitrary for #ident {
                        #implementation
                    }
                }
            } else {
                quote! {
                    impl<#(#lifetimes),*> kani::Arbitrary for #ident<#(#lifetimes),*> {
                        #implementation
                    }
                }
            }
        }
    }
}

fn parse_generics(generics: Generics) -> Vec<Lifetime> {
    let mut result: Vec<Lifetime> = vec![];
    for param in generics.params {
        if let GenericParam::Lifetime(param) = param {
            result.push(param.lifetime);
        }
    }
    result
}
