use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parenthesized, parse_macro_input, AttrStyle, Attribute, Data, DeriveInput, Type};

#[proc_macro_derive(Cuttle, attributes(cuttle))]
pub fn derive_cuttle(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    let mut build_steps: Vec<TokenStream2> = Vec::new();
    let render_data = parse_attributes(&ast.attrs, &mut build_steps);

    let render_data = match data_tokens(&mut ast, render_data) {
        Ok(render_data) => render_data,
        Err(err) => return err,
    };

    let struct_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics ::cuttle_core::prelude::Cuttle for #struct_name #type_generics #where_clause {
            fn build(mut builder: ::cuttle_core::configs::builder::CuttleBuilder<Self>) {
                builder
                .name(stringify!(#struct_name))
                #render_data
                #(
                  #build_steps
                )*
                ;
            }
        }
    })
}

fn parse_attributes(attributes: &[Attribute], steps: &mut Vec<TokenStream2>) -> Option<Type> {
    let mut result = None;

    for attr in attributes {
        let AttrStyle::Outer = attr.style else {
            continue;
        };
        if !attr.path().is_ident("cuttle") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("sort") {
                let input: TokenStream2 = meta.input.parse()?;
                steps.push(quote! { .sort #input  });
            }

            if meta.path.is_ident("extension_index_override") {
                let input: TokenStream2 = meta.input.parse()?;
                steps.push(
                    quote! { .insert(::cuttle_core::components::ExtensionIndexOverride #input ) },
                );
            }

            if meta.path.is_ident("render_data") {
                let content;
                parenthesized!(content in meta.input);
                let input: Type = content.parse()?;
                result = Some(input);
            }

            Ok(())
        })
        .unwrap();
    }

    result
}

fn data_tokens(
    ast: &DeriveInput,
    render_data: Option<Type>,
) -> Result<Option<TokenStream2>, TokenStream> {
    if let Some(render_data) = render_data {
        return Ok(Some(quote! { .render_data_from::<#render_data>() }));
    }

    Ok(match ast.data.clone() {
        Data::Struct(structure) => {
            let fields = structure.fields;

            match fields.len() {
                0 => None,
                1 => {
                    let Some(field) = fields.iter().next() else {
                        unreachable!()
                    };
                    Some(match field.ident {
                        Some(_) => quote! { .render_data() },
                        None => quote! { .render_data_deref() },
                    })
                }
                _ => Some(quote! { .render_data() }),
            }
        }
        _ => return Err(quote! { compile_error!("Only Structs are supported") }.into()),
    })
}
