use syn::{GenericArgument, PathArguments, PathSegment, Type};

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
