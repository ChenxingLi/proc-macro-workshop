use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data, DeriveInput, Field, GenericArgument, Ident, PathArguments,
    PathSegment, Type,
};

enum InputType {
    Normal(Type),
    Optional(Type),
}

impl InputType {
    fn from_input(ty: &Type) -> InputType {
        match Self::detect_option(ty) {
            Some(ty) => Self::Optional(ty),
            None => Self::Normal(ty.clone()),
        }
    }

    fn detect_option(ty: &Type) -> Option<Type> {
        if let Type::Path(path) = ty {
            if path.qself.is_some() {
                return None;
            }

            let path_segs: Vec<&PathSegment> = path.path.segments.iter().collect();
            let idents: Vec<String> = path_segs
                .iter()
                .map(|seg| format!("{}", seg.ident))
                .collect();

            let option_seg = if idents.len() == 1 && idents[0] == "Option" {
                &path_segs[0].arguments
            } else if idents.len() == 3
                && idents[0] == "std"
                && idents[1] == "option"
                && idents[2] == "Option"
            {
                &path_segs[2].arguments
            } else {
                return None;
            };

            if let PathArguments::AngleBracketed(generic_args) = option_seg {
                assert_eq!(
                    generic_args.args.len(),
                    1,
                    "std::option::Option<T> has one generic param"
                );
                if let GenericArgument::Type(ty) = generic_args.args.first().unwrap() {
                    return Some(ty.clone());
                } else {
                    unreachable!("In std::option::Option<T>, T should be a type.");
                }
            } else {
                unreachable!("std::option::Option<T> should be AngleBracketed");
            }
        }
        None
    }

    fn into_type(self) -> Type {
        match self {
            InputType::Normal(ty) => ty,
            InputType::Optional(ty) => ty,
        }
    }
}

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
        let ty = InputType::from_input(&field.ty).into_type();
        quote! { #var: ::std::option::Option<#ty>, }
    });

    let init_fields = fields.iter().cloned().map(|field| {
        let var = field.ident.clone().expect("Except named struct");
        quote! { #var: None, }
    });

    let setter = fields.iter().cloned().map(|field| {
        let var = field.ident.clone().expect("Except named struct");
        let ty = InputType::from_input(&field.ty).into_type();
        quote! {
            fn #var(&mut self, #var: #ty) -> &mut Self {
                self.#var = Some(#var);
                self
            }
        }
    });

    let clone_build_fields = fields.iter().cloned().map(|field| {
        let var = field.ident.clone().expect("Except named struct");
        match InputType::from_input(&field.ty) {
            InputType::Normal(_) => quote! {
                #var: self.#var.clone().ok_or(concat!(stringify!(#var), " has not been set."))?,
            },
            InputType::Optional(_) => quote! {
                #var: self.#var.clone(),
            },
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
