use syn::spanned::Spanned;
use syn::{
    AttrStyle, Attribute, Error, GenericArgument, Ident, Lit, Meta, MetaList, MetaNameValue,
    NestedMeta, PathArguments, PathSegment, Type,
};

pub fn detect_vec(ty: &Type) -> Option<Type> {
    if let Type::Path(path) = ty {
        if path.qself.is_some() {
            return None;
        }

        let path_segs: Vec<&PathSegment> = path.path.segments.iter().collect();
        let idents: Vec<String> = path_segs.iter().map(|seg| seg.ident.to_string()).collect();

        let option_seg = if idents.len() == 1 && idents[0] == "Vec" {
            &path_segs[0].arguments
        } else if idents.len() == 3
            && idents[0] == "std"
            && idents[1] == "vec"
            && idents[2] == "Vec"
        {
            &path_segs[2].arguments
        } else {
            return None;
        };

        if let PathArguments::AngleBracketed(generic_args) = option_seg {
            assert_eq!(
                generic_args.args.len(),
                1,
                "std::vec::Vec<T> has one generic param"
            );
            if let GenericArgument::Type(ty) = generic_args.args.first().unwrap() {
                return Some(ty.clone());
            } else {
                unreachable!("In std::vec::Vec<T>, T should be a type.");
            }
        } else {
            unreachable!("std::vec::Vec<T> should be AngleBracketed");
        }
    }
    None
}

pub fn detect_option(ty: &Type) -> Option<Type> {
    if let Type::Path(path) = ty {
        if path.qself.is_some() {
            return None;
        }

        let path_segs: Vec<&PathSegment> = path.path.segments.iter().collect();
        let idents: Vec<String> = path_segs.iter().map(|seg| seg.ident.to_string()).collect();

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

pub fn parse_builder_attribute(attr: &Attribute) -> Result<Option<Ident>, Error> {
    if matches!(attr.style, AttrStyle::Inner(_)) {
        return Ok(None);
    }

    let meta = attr.parse_meta().unwrap();

    if let Meta::List(MetaList { path, nested, .. }) = meta {
        // Check path
        if path.segments.len() != 1 {
            return Ok(None);
        }
        let attr_name = path.segments.first().unwrap();
        if !attr_name.arguments.is_empty() {
            return Ok(None);
        }

        if attr_name.ident.to_string() != "builder" {
            return Ok(None);
        }

        // Check nested
        assert_eq!(nested.len(), 1);
        let inner_meta = nested.first().unwrap();
        if let NestedMeta::Meta(Meta::NameValue(MetaNameValue {
            path,
            lit: Lit::Str(lit_str),
            ..
        })) = inner_meta
        {
            assert_eq!(path.segments.len(), 1);
            assert_eq!(
                path.segments.first().unwrap().arguments,
                PathArguments::None
            );
            let attr_key = &path.segments.first().unwrap().ident;
            if attr_key.to_string() != "each" {
                return Err(Error::new(
                    attr.tokens.span().join(attr.path.span()).unwrap(),
                    "expected `builder(each = \"...\")`",
                ));
            }

            return Ok(Some(Ident::new(&lit_str.value(), lit_str.span())));
        } else {
            unreachable!()
        }
    } else {
        unreachable!()
    }
}
