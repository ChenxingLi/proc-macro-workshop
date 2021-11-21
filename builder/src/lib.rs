mod parse_type;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::convert::{TryFrom, TryInto};
use syn::{parse_macro_input, Data, DeriveInput, Error, Field, Ident, Type};

use crate::parse_type::{detect_option, detect_vec, parse_builder_attribute};

struct InputField {
    var: Ident,
    ty: Type,
    is_option: bool,
    vec_setter: Option<Ident>,
}

impl TryFrom<&Field> for InputField {
    type Error = Error;
    fn try_from(field: &Field) -> Result<Self, Error> {
        let var = field.ident.clone().expect("Except named struct");

        let mut is_option = false;
        let mut ty = field.ty.clone();

        if let Some(option_ty) = detect_option(&ty) {
            is_option = true;
            ty = option_ty;
        }

        let mut iter = field.attrs.iter();
        let vec_setter = loop {
            if let Some(attr) = iter.next() {
                if let Some(ident) = parse_builder_attribute(attr)? {
                    break Some(ident);
                }
            } else {
                break None;
            }
        };

        if vec_setter.is_some() {
            ty = detect_vec(&ty).expect("Type should be Vec if 'each' is set");
        }

        Ok(Self {
            var,
            ty,
            is_option,
            vec_setter,
        })
    }
}

impl InputField {
    fn builder_fields(&self) -> TokenStream2 {
        let var = &self.var;
        let ty = &self.ty;
        if self.vec_setter.is_some() {
            quote! { #var: ::std::vec::Vec<#ty>, }
        } else {
            quote! { #var: ::std::option::Option<#ty>, }
        }
    }

    fn builder_init(&self) -> TokenStream2 {
        let var = &self.var;
        if self.vec_setter.is_some() {
            quote! { #var: ::std::vec::Vec::new(), }
        } else {
            quote! { #var: None, }
        }
    }

    fn setter_fns(&self) -> TokenStream2 {
        let var = &self.var;
        let ty = &self.ty;
        if let Some(setter) = &self.vec_setter {
            let input = Ident::new(&format!("{}_item", var), var.span());
            quote! {
                fn #setter(&mut self, #input: #ty) -> &mut Self {
                    self.#var.push(#input);
                    self
                }
            }
        } else {
            quote! {
                fn #var(&mut self, #var: #ty) -> &mut Self {
                    self.#var = Some(#var);
                    self
                }
            }
        }
    }

    fn clone_to_struct(&self) -> TokenStream2 {
        let var = &self.var;
        if self.is_option || self.vec_setter.is_some() {
            quote! { #var: self.#var.clone(), }
        } else {
            quote! { #var: self.#var.clone().ok_or(concat!(stringify!(#var), " has not been set."))?, }
        }
    }
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    // eprint!("{:#?}", ast);

    let vis = &ast.vis;
    let ident = &ast.ident;
    let bident = Ident::new(&format!("{}Builder", ident), ident.span());

    let parse_fields = || -> Result<Vec<InputField>, Error> {
        if let Data::Struct(ref data_struct) = ast.data {
            let mut parse_fields = Vec::with_capacity(data_struct.fields.len());
            for x in data_struct.fields.iter() {
                parse_fields.push(x.try_into()?);
            }
            Ok(parse_fields)
        } else {
            unimplemented!("Only support struct");
        }
    };

    let fields = match parse_fields() {
        Ok(fields) => fields,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let builder_fields = fields.iter().map(InputField::builder_fields);
    let init_fields = fields.iter().map(InputField::builder_init);
    let setter_fns = fields.iter().map(InputField::setter_fns);
    let clone_to_struct = fields.iter().map(InputField::clone_to_struct);

    let build_struct = quote! {
        #vis struct #bident {
            #(#builder_fields)*
        }

        impl #ident {
            pub fn builder() -> #bident {
                #bident {
                    #(#init_fields)*
                }
            }
        }

        impl #bident {
            pub fn build(&mut self) -> ::std::result::Result<#ident, ::std::boxed::Box<dyn ::std::error::Error>> {
                Ok(#ident {
                    #(#clone_to_struct)*
                })
            }
            #(#setter_fns)*
        }
    };
    build_struct.into()
}
