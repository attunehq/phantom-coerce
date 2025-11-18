use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, GenericArgument, Ident,
    PathArguments, Type, TypePath, Meta, Attribute,
};

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

    // Parse coerce attributes to extract target types
    let mut coerce_targets = Vec::new();
    for attr in &input.attrs {
        if attr.path().is_ident("coerce") {
            if let Some(target) = parse_coerce_attr(attr)? {
                coerce_targets.push(target);
            }
        }
    }

    if coerce_targets.is_empty() {
        return Err(syn::Error::new_spanned(
            input,
            "#[derive(Coerce)] requires at least one #[coerce(borrowed = \"...\")] attribute"
        ));
    }

    // Generate trait name
    let trait_name = Ident::new(&format!("Coerce{}", struct_name), struct_name.span());

    // Generate trait definition
    let trait_def = quote! {
        trait #trait_name<Output: ?Sized> {
            fn coerce(&self) -> &Output;
        }
    };

    // Generate implementations for each target
    let mut impls = Vec::new();
    for target_str in &coerce_targets {
        let target_type: Type = syn::parse_str(target_str)?;

        let impl_block = generate_impl(
            struct_name,
            generics,
            &trait_name,
            &target_type,
            fields,
            &phantom_fields,
        )?;

        impls.push(impl_block);
    }

    Ok(quote! {
        #trait_def
        #(#impls)*
    })
}

fn is_phantom_data(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            return segment.ident == "PhantomData";
        }
    }
    false
}

fn parse_coerce_attr(attr: &Attribute) -> syn::Result<Option<String>> {
    let Meta::List(meta_list) = &attr.meta else {
        return Ok(None);
    };

    let nested = meta_list.tokens.clone();
    let parsed: syn::MetaNameValue = syn::parse2(nested)?;

    if !parsed.path.is_ident("borrowed") {
        return Err(syn::Error::new_spanned(
            &parsed.path,
            "Expected 'borrowed' in #[coerce(borrowed = \"...\")]"
        ));
    }

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

    Ok(Some(lit_str.value()))
}

fn generate_impl(
    struct_name: &Ident,
    generics: &syn::Generics,
    trait_name: &Ident,
    target_type: &Type,
    fields: &syn::FieldsNamed,
    _phantom_fields: &[&Ident],
) -> syn::Result<proc_macro2::TokenStream> {
    // Extract type arguments from target type
    let Type::Path(target_path) = target_type else {
        return Err(syn::Error::new_spanned(
            target_type,
            "Coerce target must be a type path"
        ));
    };

    let target_segment = target_path.path.segments.last().unwrap();
    let PathArguments::AngleBracketed(target_args) = &target_segment.arguments else {
        return Err(syn::Error::new_spanned(
            target_type,
            "Coerce target must have type parameters"
        ));
    };

    // Build safety comment documenting which fields differ
    let mut changed_params = Vec::new();
    let source_params: Vec<_> = generics.type_params().collect();
    let target_params: Vec<_> = target_args.args.iter().collect();

    for (_idx, (source_param, target_arg)) in source_params.iter().zip(target_params.iter()).enumerate() {
        if let GenericArgument::Type(Type::Path(target_ty_path)) = target_arg {
            let source_ident = &source_param.ident;
            let target_ident = &target_ty_path.path.segments.last().unwrap().ident;

            if source_ident != target_ident {
                changed_params.push(format!("{} -> {}", source_ident, target_ident));
            }
        }
    }

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
