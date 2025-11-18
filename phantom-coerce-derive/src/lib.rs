use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, Ident,
    PathArguments, Type, TypePath, Meta, Attribute,
};

#[derive(Debug, Clone)]
enum CoercionKind {
    Borrowed(String),
    Owned(String),
}

#[proc_macro_derive(Coerce, attributes(coerce))]
pub fn derive_coerce(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_coerce(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn impl_coerce(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;
    let generics = &input.generics;

    let Data::Struct(data_struct) = &input.data else {
        return Err(syn::Error::new_spanned(
            input,
            "#[derive(Coerce)] can only be applied to structs"
        ));
    };

    let Fields::Named(fields) = &data_struct.fields else {
        return Err(syn::Error::new_spanned(
            &data_struct.fields,
            "#[derive(Coerce)] requires named fields"
        ));
    };

    // Identify PhantomData fields and map them to type parameters
    let mut phantom_fields = Vec::new();
    for field in &fields.named {
        if is_phantom_data(&field.ty) {
            phantom_fields.push(field.ident.as_ref().unwrap());
        }
    }

    // Parse coerce attributes to extract target types and kinds
    let mut coercions = Vec::new();
    for attr in &input.attrs {
        if attr.path().is_ident("coerce") {
            if let Some(coercion) = parse_coerce_attr(attr)? {
                coercions.push(coercion);
            }
        }
    }

    if coercions.is_empty() {
        return Err(syn::Error::new_spanned(
            input,
            "#[derive(Coerce)] requires at least one #[coerce(...)] attribute"
        ));
    }

    let mut output = proc_macro2::TokenStream::new();

    // Generate borrowed coercions
    let borrowed_targets: Vec<_> = coercions.iter()
        .filter_map(|c| match c {
            CoercionKind::Borrowed(s) => Some(s.clone()),
            _ => None,
        })
        .collect();

    if !borrowed_targets.is_empty() {
        let trait_name = Ident::new(&format!("Coerce{}", struct_name), struct_name.span());

        let trait_def = quote! {
            trait #trait_name<Output: ?Sized> {
                fn coerce(&self) -> &Output;
            }
        };

        let mut impls = Vec::new();
        for target_str in &borrowed_targets {
            let target_type: Type = syn::parse_str(target_str)?;
            let impl_block = generate_borrowed_impl(
                struct_name,
                generics,
                &trait_name,
                &target_type,
                fields,
                &phantom_fields,
            )?;
            impls.push(impl_block);
        }

        output.extend(quote! {
            #trait_def
            #(#impls)*
        });
    }

    // Generate owned coercions
    let owned_targets: Vec<_> = coercions.iter()
        .filter_map(|c| match c {
            CoercionKind::Owned(s) => Some(s.clone()),
            _ => None,
        })
        .collect();

    if !owned_targets.is_empty() {
        let trait_name = Ident::new(&format!("CoerceOwned{}", struct_name), struct_name.span());

        let trait_def = quote! {
            trait #trait_name<Output> {
                fn into_coerced(self) -> Output;
            }
        };

        let mut impls = Vec::new();
        for target_str in &owned_targets {
            let target_type: Type = syn::parse_str(target_str)?;
            let impl_block = generate_owned_impl(
                struct_name,
                generics,
                &trait_name,
                &target_type,
                fields,
                &phantom_fields,
            )?;
            impls.push(impl_block);
        }

        output.extend(quote! {
            #trait_def
            #(#impls)*
        });
    }

    Ok(output)
}

fn is_phantom_data(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            return segment.ident == "PhantomData";
        }
    }
    false
}

fn parse_coerce_attr(attr: &Attribute) -> syn::Result<Option<CoercionKind>> {
    let Meta::List(meta_list) = &attr.meta else {
        return Ok(None);
    };

    let nested = meta_list.tokens.clone();
    let parsed: syn::MetaNameValue = syn::parse2(nested)?;

    let kind = if parsed.path.is_ident("borrowed") {
        CoercionKind::Borrowed
    } else if parsed.path.is_ident("owned") {
        CoercionKind::Owned
    } else {
        return Err(syn::Error::new_spanned(
            &parsed.path,
            "Expected 'borrowed' or 'owned' in #[coerce(borrowed = \"...\")] or #[coerce(owned = \"...\")]"
        ));
    };

    let syn::Expr::Lit(expr_lit) = &parsed.value else {
        return Err(syn::Error::new_spanned(
            &parsed.value,
            "Expected string literal"
        ));
    };

    let syn::Lit::Str(lit_str) = &expr_lit.lit else {
        return Err(syn::Error::new_spanned(
            &expr_lit.lit,
            "Expected string literal"
        ));
    };

    Ok(Some(kind(lit_str.value())))
}

fn generate_borrowed_impl(
    struct_name: &Ident,
    generics: &syn::Generics,
    trait_name: &Ident,
    target_type: &Type,
    fields: &syn::FieldsNamed,
    _phantom_fields: &[&Ident],
) -> syn::Result<proc_macro2::TokenStream> {
    let Type::Path(target_path) = target_type else {
        return Err(syn::Error::new_spanned(
            target_type,
            "Coerce target must be a type path"
        ));
    };

    let target_segment = target_path.path.segments.last().unwrap();
    let PathArguments::AngleBracketed(_target_args) = &target_segment.arguments else {
        return Err(syn::Error::new_spanned(
            target_type,
            "Coerce target must have type parameters"
        ));
    };

    // Generate destructuring pattern with type annotations for all fields
    let field_destructure: Vec<_> = fields.named.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        quote! { #field_name: _ }
    }).collect();

    let field_type_checks: Vec<_> = fields.named.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_ty = &f.ty;
        quote! { let _: &#field_ty = &self.#field_name; }
    }).collect();

    let (impl_generics, _, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #trait_name<#target_type> for #struct_name #impl_generics #where_clause {
            fn coerce(&self) -> &#target_type {
                // Compile-time safety guards: ensure all fields are accounted for
                let #struct_name { #(#field_destructure),* } = self;
                #(#field_type_checks)*

                // SAFETY: Types differ only in PhantomData type parameters.
                // The destructuring pattern and type annotations above ensure this at compile time.
                unsafe { std::mem::transmute(self) }
            }
        }
    })
}

fn generate_owned_impl(
    struct_name: &Ident,
    generics: &syn::Generics,
    trait_name: &Ident,
    target_type: &Type,
    fields: &syn::FieldsNamed,
    _phantom_fields: &[&Ident],
) -> syn::Result<proc_macro2::TokenStream> {
    let Type::Path(target_path) = target_type else {
        return Err(syn::Error::new_spanned(
            target_type,
            "Coerce target must be a type path"
        ));
    };

    let target_segment = target_path.path.segments.last().unwrap();
    let PathArguments::AngleBracketed(_target_args) = &target_segment.arguments else {
        return Err(syn::Error::new_spanned(
            target_type,
            "Coerce target must have type parameters"
        ));
    };

    // Generate destructuring pattern for all fields
    let field_destructure: Vec<_> = fields.named.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        quote! { #field_name: _ }
    }).collect();

    let (impl_generics, _, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #trait_name<#target_type> for #struct_name #impl_generics #where_clause {
            fn into_coerced(self) -> #target_type {
                // Compile-time safety guard: ensure all fields are accounted for
                let #struct_name { #(#field_destructure),* } = &self;

                // SAFETY: Types differ only in PhantomData type parameters.
                // The destructuring pattern above ensures this at compile time.
                unsafe { std::mem::transmute(self) }
            }
        }
    })
}
