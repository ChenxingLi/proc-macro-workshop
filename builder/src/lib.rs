use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Ident};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    // eprint!("{:#?}", ast);

    let vis = &ast.vis;
    let ident = &ast.ident;
    let bident = Ident::new(&format!("{}Builder", ident), ident.span());

    let fields: Vec<&Field> = if let Data::Struct(ref data_struct) = ast.data {
        data_struct.fields.iter().collect()
    } else {
        unimplemented!("Only support struct");
    };

    let optionized_fields = fields.iter().cloned().map(|field| {
        let var = field.ident.clone().expect("Except named struct");
        let ty = field.ty.clone();
        quote! { #var: ::std::option::Option<#ty>, }
    });

    let init_fields = fields.iter().cloned().map(|field| {
        let var = field.ident.clone().expect("Except named struct");
        quote! { #var: None, }
    });

    let setter = fields.iter().cloned().map(|field| {
        let var = field.ident.clone().expect("Except named struct");
        let ty = field.ty.clone();
        quote! {
            fn #var(&mut self, #var: #ty) -> &mut Self {
                self.#var = Some(#var);
                self
            }
        }
    });

    let clone_build_fields = fields.iter().cloned().map(|field| {
        let var = field.ident.clone().expect("Except named struct");
        quote! {
            #var: self.#var.clone().ok_or(concat!(stringify!(#var), " has not been set."))?,
        }
    });

    let build_struct = quote! {
        #vis struct #bident {
            #(#optionized_fields)*
        }

        impl #ident {
            pub fn builder() -> #bident {
                #bident {
                    #(#init_fields)*
                }
            }
        }

        impl #bident {
            pub fn build(&mut self) -> ::std::result::Result<#ident, Box<dyn ::std::error::Error>> {
                Ok(#ident {
                    #(#clone_build_fields)*
                })
            }
            #(#setter)*
        }
    };
    build_struct.into()
}
