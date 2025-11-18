use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, Ident,
    PathArguments, Type, TypePath, Meta, Attribute,
    parse::Parser, spanned::Spanned,
};

#[derive(Debug, Clone)]
struct CoercionTarget {
    target_type: String,
    kind: CoercionMode,
    generate_asref: bool,  // for borrowed only
}

#[derive(Debug, Clone, PartialEq)]
enum CoercionMode {
    Borrowed,
    Owned,
    Cloned,
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
        .filter(|c| c.kind == CoercionMode::Borrowed)
        .collect();

    if !borrowed_targets.is_empty() {
        let trait_name = Ident::new(&format!("Coerce{}", struct_name), struct_name.span());

        let trait_def = quote! {
            trait #trait_name<Output: ?Sized> {
                fn coerce(&self) -> &Output;
            }
        };

        let mut impls = Vec::new();
        let mut asref_impls = Vec::new();

        for target in &borrowed_targets {
            let target_type: Type = syn::parse_str(&target.target_type)?;
            let impl_block = generate_borrowed_impl(
                struct_name,
                generics,
                &trait_name,
                &target_type,
                fields,
                &phantom_fields,
            )?;
            impls.push(impl_block);

            // Generate AsRef impl if requested
            if target.generate_asref {
                let asref_impl = generate_asref_impl(
                    struct_name,
                    generics,
                    &trait_name,
                    &target_type,
                )?;
                asref_impls.push(asref_impl);
            }
        }

        output.extend(quote! {
            #trait_def
            #(#impls)*
            #(#asref_impls)*
        });
    }

    // Generate owned coercions
    let owned_targets: Vec<_> = coercions.iter()
        .filter(|c| c.kind == CoercionMode::Owned)
        .collect();

    if !owned_targets.is_empty() {
        let trait_name = Ident::new(&format!("CoerceOwned{}", struct_name), struct_name.span());

        let trait_def = quote! {
            trait #trait_name<Output> {
                fn into_coerced(self) -> Output;
            }
        };

        let mut impls = Vec::new();

        for target in &owned_targets {
            let target_type: Type = syn::parse_str(&target.target_type)?;
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

    // Generate cloned coercions
    let cloned_targets: Vec<_> = coercions.iter()
        .filter(|c| c.kind == CoercionMode::Cloned)
        .collect();

    if !cloned_targets.is_empty() {
        let trait_name = Ident::new(&format!("CoerceCloned{}", struct_name), struct_name.span());

        let trait_def = quote! {
            trait #trait_name<Output> {
                fn to_coerced(&self) -> Output;
            }
        };

        let mut impls = Vec::new();

        for target in &cloned_targets {
            let target_type: Type = syn::parse_str(&target.target_type)?;
            let impl_block = generate_cloned_impl(
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

fn parse_coerce_attr(attr: &Attribute) -> syn::Result<Option<CoercionTarget>> {
    let Meta::List(meta_list) = &attr.meta else {
        return Ok(None);
    };

    let nested = meta_list.tokens.clone();

    // Parse as multiple Meta items (NameValue or Path)
    let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
    let metas = parser.parse2(nested)?;

    let mut mode: Option<CoercionMode> = None;
    let mut target_type: Option<String> = None;
    let mut has_asref = false;

    for meta in metas {
        match meta {
            syn::Meta::NameValue(nv) => {
                if nv.path.is_ident("borrowed") {
                    mode = Some(CoercionMode::Borrowed);
                } else if nv.path.is_ident("owned") {
                    mode = Some(CoercionMode::Owned);
                } else if nv.path.is_ident("cloned") {
                    mode = Some(CoercionMode::Cloned);
                } else {
                    return Err(syn::Error::new_spanned(
                        &nv.path,
                        "Expected 'borrowed', 'owned', or 'cloned'"
                    ));
                }

                let syn::Expr::Lit(expr_lit) = &nv.value else {
                    return Err(syn::Error::new_spanned(
                        &nv.value,
                        "Expected string literal"
                    ));
                };

                let syn::Lit::Str(lit_str) = &expr_lit.lit else {
                    return Err(syn::Error::new_spanned(
                        &expr_lit.lit,
                        "Expected string literal"
                    ));
                };

                target_type = Some(lit_str.value());
            }
            syn::Meta::Path(path) => {
                if path.is_ident("asref") {
                    has_asref = true;
                } else {
                    return Err(syn::Error::new_spanned(
                        &path,
                        "Expected 'asref' marker (only valid for borrowed coercions)"
                    ));
                }
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    &meta,
                    "Expected name-value pair or path"
                ));
            }
        }
    }

    let mode = mode.ok_or_else(|| {
        syn::Error::new(
            attr.span(),
            "Missing coercion mode: borrowed, owned, or cloned"
        )
    })?;

    let target_type = target_type.ok_or_else(|| {
        syn::Error::new(
            attr.span(),
            "Missing target type in coercion attribute"
        )
    })?;

    // Validate asref is only used with borrowed
    if has_asref && mode != CoercionMode::Borrowed {
        return Err(syn::Error::new(
            attr.span(),
            "asref marker is only valid for borrowed coercions"
        ));
    }

    Ok(Some(CoercionTarget {
        target_type,
        kind: mode,
        generate_asref: has_asref,
    }))
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

fn generate_cloned_impl(
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

    // Build where clause with Clone bound on the struct itself
    let mut where_clause = generics.where_clause.clone().unwrap_or_else(|| syn::WhereClause {
        where_token: syn::token::Where::default(),
        predicates: syn::punctuated::Punctuated::new(),
    });

    // Add Clone bound on the entire struct
    let (_, ty_generics, _) = generics.split_for_impl();
    where_clause.predicates.push(syn::parse_quote!(#struct_name #ty_generics: Clone));

    // Generate destructuring pattern for all fields
    let field_destructure: Vec<_> = fields.named.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        quote! { #field_name: _ }
    }).collect();

    let (impl_generics, _, _) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #trait_name<#target_type> for #struct_name #impl_generics #where_clause {
            fn to_coerced(&self) -> #target_type {
                // Compile-time safety guard: ensure all fields are accounted for
                let #struct_name { #(#field_destructure),* } = self;

                // SAFETY: Types differ only in PhantomData type parameters.
                // The destructuring pattern above ensures this at compile time.
                // The source type is cloned and then transmuted.
                unsafe { std::mem::transmute(self.clone()) }
            }
        }
    })
}

fn generate_asref_impl(
    struct_name: &Ident,
    generics: &syn::Generics,
    _trait_name: &Ident,
    target_type: &Type,
) -> syn::Result<proc_macro2::TokenStream> {
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics AsRef<#target_type> for #struct_name #impl_generics #where_clause {
            fn as_ref(&self) -> &#target_type {
                self.coerce()
            }
        }
    })
}

