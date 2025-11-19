use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Fields, Ident, Meta, PathArguments, Type, TypePath,
    parse::Parser, parse_macro_input, spanned::Spanned,
};

#[derive(Debug, Clone)]
struct CoercionSpec {
    /// Source type patterns (parsed from `borrowed_from`, `owned_from`, `cloned_from`)
    /// Each string may contain `|` for multiple alternatives like "Absolute | Relative"
    from_patterns: Vec<String>,
    /// Target type pattern (parsed from `borrowed_to`, `owned_to`, `cloned_to`)
    to_pattern: String,
    kind: CoercionMode,
    generate_asref: bool, // for borrowed only
}

#[derive(Debug, Clone)]
struct ParsedCoercion {
    /// Source type with type holes resolved to generic parameters
    source_type: Type,
    /// Target type with type holes resolved to generic parameters
    target_type: Type,
    /// Indices of type parameters that should be preserved (type holes)
    type_hole_positions: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq)]
enum CoercionMode {
    Borrowed,
    Owned,
    Cloned,
}

/// Derive macro for safe, zero-cost coercion between types differing only in PhantomData parameters.
///
/// # Coercion Modes
///
/// - `#[coerce(borrowed = "Target")]`: Generate `coerce(&self) -> &Target` method
/// - `#[coerce(owned = "Target")]`: Generate `into_coerced(self) -> Target` method
/// - `#[coerce(cloned = "Target")]`: Generate `to_coerced(&self) -> Target` method (requires Clone)
///
/// # Multiple Target Types with `|` Syntax
///
/// Use the `|` operator to specify multiple source or target types. This works at two levels:
///
/// **1. Top-level alternatives (between complete types):**
/// ```ignore
/// #[coerce(borrowed_from = "Container<TypeA>", borrowed_to = "Container<Generic> | Container<AnotherGeneric>")]
/// ```
/// Generates two coercions: `Container<TypeA>` → `Container<Generic>` and `Container<TypeA>` → `Container<AnotherGeneric>`
///
/// **2. Parameter-level alternatives (within type parameters):**
/// ```ignore
/// #[coerce(borrowed_from = "TypedPath<Absolute | Relative, File>", borrowed_to = "TypedPath<UnknownBase, File>")]
/// ```
/// Generates Cartesian product: `TypedPath<Absolute, File>` → `TypedPath<UnknownBase, File>`
/// and `TypedPath<Relative, File>` → `TypedPath<UnknownBase, File>`
///
/// Both syntaxes work on both `_from` and `_to` sides.
///
/// # Type Hole Syntax
///
/// Use `_` in type parameters to preserve specific parameters during coercion:
/// - `#[coerce(borrowed = "Type<SomeType, _>")]`: Coerce first parameter, preserve second
/// - `#[coerce(borrowed = "Type<_, OtherType>")]`: Preserve first parameter, coerce second
///
/// Type holes prevent unintended cross-parameter coercions by ensuring only specified
/// parameters change while others remain identical.
///
/// # Optional Markers
///
/// - `asref`: For borrowed coercions, also generate `AsRef<Target>` implementation
///   - Example: `#[coerce(borrowed = "Type<T>", asref)]`
///
/// # Turbofish Support
///
/// All methods support turbofish syntax for explicit type specification:
/// - `.coerce::<Target>()` instead of needing type annotations
/// - `.into_coerced::<Target>()`
/// - `.to_coerced::<Target>()`
///
/// # Examples
///
/// ```rust,ignore
/// use std::marker::PhantomData;
/// use phantom_coerce::Coerce;
///
/// // Type markers for path bases (specific -> generic)
/// struct Absolute;
/// struct Relative;
/// struct UnknownBase;  // Generic base (subsumes Absolute and Relative)
///
/// // Type markers for path types (specific -> generic)
/// struct File;
/// struct Directory;
/// struct UnknownType;  // Generic type (subsumes File and Directory)
///
/// #[derive(Coerce, Clone)]
/// #[coerce(borrowed = "TypedPath<UnknownBase, UnknownType>", asref)]  // Coerce both params to generic
/// #[coerce(owned = "TypedPath<Absolute, UnknownType>")]  // Coerce just type param to generic
/// #[coerce(cloned = "TypedPath<UnknownBase, File>")]  // Coerce just base param to generic
/// struct TypedPath<Base, Type> {
///     base: PhantomData<Base>,
///     ty: PhantomData<Type>,
///     path: String,
/// }
///
/// fn main() {
///     let path = TypedPath::<Absolute, File> {
///         base: PhantomData,
///         ty: PhantomData,
///         path: "/home/user/file.txt".to_string(),
///     };
///
///     // Borrowed: coerce to more generic type (both params)
///     let r1: &TypedPath<UnknownBase, UnknownType> = path.coerce();
///     let r2 = path.coerce::<TypedPath<UnknownBase, UnknownType>>();
///
///     // AsRef: works because we added the asref marker
///     let r3: &TypedPath<UnknownBase, UnknownType> = path.as_ref();
///
///     // Owned: coerce type param to generic (consumes path)
///     let path2 = TypedPath::<Absolute, File> {
///         base: PhantomData,
///         ty: PhantomData,
///         path: "/test".to_string(),
///     };
///     let owned: TypedPath<Absolute, UnknownType> = path2.into_coerced();
///
///     // Cloned: coerce base param to generic (path remains usable)
///     let cloned = path.to_coerced::<TypedPath<UnknownBase, File>>();
/// }
/// ```
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
            "#[derive(Coerce)] can only be applied to structs",
        ));
    };

    let Fields::Named(fields) = &data_struct.fields else {
        return Err(syn::Error::new_spanned(
            &data_struct.fields,
            "#[derive(Coerce)] requires named fields",
        ));
    };

    // Identify PhantomData fields and map them to type parameters
    let mut phantom_fields = Vec::new();
    for field in &fields.named {
        if is_phantom_data(&field.ty) {
            phantom_fields.push(field.ident.as_ref().unwrap());
        }
    }

    // Parse coerce attributes and expand into concrete coercion instances
    let mut coercion_specs = Vec::new();
    for attr in &input.attrs {
        if attr.path().is_ident("coerce")
            && let Some(spec) = parse_coerce_attr(attr)?
        {
            coercion_specs.push(spec);
        }
    }

    if coercion_specs.is_empty() {
        return Err(syn::Error::new_spanned(
            input,
            "#[derive(Coerce)] requires at least one #[coerce(...)] attribute",
        ));
    }

    // Expand all specs into concrete coercions
    let mut borrowed_coercions = Vec::new();
    let mut owned_coercions = Vec::new();
    let mut cloned_coercions = Vec::new();
    let mut generate_asref_for = Vec::new();

    for spec in &coercion_specs {
        let expanded = expand_coercion_spec(spec, generics)?;
        match spec.kind {
            CoercionMode::Borrowed => {
                borrowed_coercions.extend(expanded);
                if spec.generate_asref {
                    // Mark which coercions should also generate AsRef
                    generate_asref_for.extend((0..borrowed_coercions.len()).collect::<Vec<_>>());
                }
            }
            CoercionMode::Owned => owned_coercions.extend(expanded),
            CoercionMode::Cloned => cloned_coercions.extend(expanded),
        }
    }

    let mut output = proc_macro2::TokenStream::new();

    // Generate borrowed coercions
    if !borrowed_coercions.is_empty() {
        let trait_name = Ident::new(&format!("CoerceRef{}", struct_name), struct_name.span());

        let trait_def = quote! {
            trait #trait_name<Output: ?Sized> {
                fn coerce(&self) -> &Output;
            }
        };

        let mut impls = Vec::new();
        let mut asref_impls = Vec::new();

        for (idx, coercion) in borrowed_coercions.iter().enumerate() {
            let impl_block = generate_borrowed_impl(
                struct_name,
                generics,
                &trait_name,
                coercion,
                fields,
                &phantom_fields,
            )?;
            impls.push(impl_block);

            // Generate AsRef impl if this coercion was marked for it
            if generate_asref_for.contains(&idx) {
                let asref_impl = generate_asref_impl(struct_name, generics, &trait_name, coercion)?;
                asref_impls.push(asref_impl);
            }
        }

        // Generate inherent method with turbofish support
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let inherent_method = quote! {
            impl #impl_generics #struct_name #ty_generics #where_clause {
                fn coerce<__CoerceTarget>(&self) -> &__CoerceTarget
                where
                    Self: #trait_name<__CoerceTarget>,
                    __CoerceTarget: ?Sized,
                {
                    #trait_name::coerce(self)
                }
            }
        };

        output.extend(quote! {
            #trait_def
            #(#impls)*
            #inherent_method
            #(#asref_impls)*
        });
    }

    // Generate owned coercions
    if !owned_coercions.is_empty() {
        let trait_name = Ident::new(&format!("CoerceOwned{}", struct_name), struct_name.span());

        let trait_def = quote! {
            trait #trait_name<Output> {
                fn into_coerced(self) -> Output;
            }
        };

        let mut impls = Vec::new();

        for coercion in &owned_coercions {
            let impl_block = generate_owned_impl(
                struct_name,
                generics,
                &trait_name,
                coercion,
                fields,
                &phantom_fields,
            )?;
            impls.push(impl_block);
        }

        // Generate inherent method with turbofish support
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let inherent_method = quote! {
            impl #impl_generics #struct_name #ty_generics #where_clause {
                fn into_coerced<__CoerceTarget>(self) -> __CoerceTarget
                where
                    Self: #trait_name<__CoerceTarget>,
                    __CoerceTarget: Sized,
                {
                    #trait_name::into_coerced(self)
                }
            }
        };

        output.extend(quote! {
            #trait_def
            #(#impls)*
            #inherent_method
        });
    }

    // Generate cloned coercions
    if !cloned_coercions.is_empty() {
        let trait_name = Ident::new(&format!("CoerceCloned{}", struct_name), struct_name.span());

        let trait_def = quote! {
            trait #trait_name<Output> {
                fn to_coerced(&self) -> Output;
            }
        };

        let mut impls = Vec::new();

        for coercion in &cloned_coercions {
            let impl_block = generate_cloned_impl(
                struct_name,
                generics,
                &trait_name,
                coercion,
                fields,
                &phantom_fields,
            )?;
            impls.push(impl_block);
        }

        // Generate inherent method with turbofish support
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let inherent_method = quote! {
            impl #impl_generics #struct_name #ty_generics #where_clause {
                fn to_coerced<__CoerceTarget>(&self) -> __CoerceTarget
                where
                    Self: #trait_name<__CoerceTarget>,
                    __CoerceTarget: Sized,
                {
                    #trait_name::to_coerced(self)
                }
            }
        };

        output.extend(quote! {
            #trait_def
            #(#impls)*
            #inherent_method
        });
    }

    Ok(output)
}

fn is_phantom_data(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty
        && let Some(segment) = path.segments.last()
    {
        return segment.ident == "PhantomData";
    }
    false
}

#[derive(Debug, Clone)]
struct ParsedPattern {
    /// The type with type holes resolved to generic parameters
    target_type: Type,
    /// Indices of type parameters that should be preserved (type holes)
    type_hole_positions: Vec<usize>,
}

/// Parse target type string, extracting type hole positions and resolving them
fn parse_target_with_type_holes(
    target_str: &str,
    generics: &syn::Generics,
) -> syn::Result<ParsedPattern> {
    // Check if contains type holes by looking for standalone _ in type arguments
    let has_type_hole =
        target_str.contains("<_") || target_str.contains(", _") || target_str.contains("_>");

    if !has_type_hole {
        // No type holes, parse normally
        let target_type: Type = syn::parse_str(target_str)?;
        return Ok(ParsedPattern {
            target_type,
            type_hole_positions: Vec::new(),
        });
    }

    // Get the generic parameter names
    let params: Vec<&Ident> = generics
        .params
        .iter()
        .filter_map(|p| {
            if let syn::GenericParam::Type(tp) = p {
                Some(&tp.ident)
            } else {
                None
            }
        })
        .collect();

    // Parse by splitting on angle brackets and commas
    let mut type_hole_positions = Vec::new();
    let mut resolved_target = String::new();
    let mut param_index = 0;
    let mut in_angle_brackets = false;
    let mut current_token = String::new();

    for ch in target_str.chars() {
        match ch {
            '<' => {
                // Push accumulated struct name before the angle bracket
                if !current_token.is_empty() {
                    resolved_target.push_str(&current_token);
                    current_token.clear();
                }
                resolved_target.push(ch);
                in_angle_brackets = true;
                param_index = 0;
            }
            '>' => {
                if !current_token.is_empty() {
                    if current_token.trim() == "_" {
                        type_hole_positions.push(param_index);
                        if param_index < params.len() {
                            resolved_target.push_str(&params[param_index].to_string());
                        } else {
                            return Err(syn::Error::new(
                                proc_macro2::Span::call_site(),
                                format!(
                                    "Type hole at position {} but struct only has {} type parameters",
                                    param_index,
                                    params.len()
                                ),
                            ));
                        }
                    } else {
                        resolved_target.push_str(&current_token);
                    }
                    current_token.clear();
                }
                resolved_target.push(ch);
                in_angle_brackets = false;
            }
            ',' if in_angle_brackets => {
                if !current_token.is_empty() {
                    if current_token.trim() == "_" {
                        type_hole_positions.push(param_index);
                        if param_index < params.len() {
                            resolved_target.push_str(&params[param_index].to_string());
                        } else {
                            return Err(syn::Error::new(
                                proc_macro2::Span::call_site(),
                                format!(
                                    "Type hole at position {} but struct only has {} type parameters",
                                    param_index,
                                    params.len()
                                ),
                            ));
                        }
                    } else {
                        resolved_target.push_str(&current_token);
                    }
                    current_token.clear();
                }
                resolved_target.push(ch);
                resolved_target.push(' ');
                param_index += 1;
            }
            _ => {
                current_token.push(ch);
            }
        }
    }

    // Handle any remaining token (for non-generic types at the end)
    if !current_token.is_empty() {
        resolved_target.push_str(&current_token);
    }

    let target_type: Type = syn::parse_str(&resolved_target).map_err(|e| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!(
                "Failed to parse resolved target '{}': {}",
                resolved_target, e
            ),
        )
    })?;

    Ok(ParsedPattern {
        target_type,
        type_hole_positions,
    })
}

fn parse_coerce_attr(attr: &Attribute) -> syn::Result<Option<CoercionSpec>> {
    let Meta::List(meta_list) = &attr.meta else {
        return Ok(None);
    };

    let nested = meta_list.tokens.clone();

    // Parse as multiple Meta items (NameValue or Path)
    let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
    let metas = parser.parse2(nested)?;

    let mut mode: Option<CoercionMode> = None;
    let mut from_patterns: Vec<String> = Vec::new();
    let mut to_pattern: Option<String> = None;
    let mut has_asref = false;
    let mut from_mode_seen: Option<CoercionMode> = None;
    let mut to_mode_seen: Option<CoercionMode> = None;

    for meta in metas {
        match meta {
            syn::Meta::NameValue(nv) => {
                // Parse borrowed_from/to, owned_from/to, cloned_from/to
                if nv.path.is_ident("borrowed_from") {
                    mode = Some(CoercionMode::Borrowed);
                    from_mode_seen = Some(CoercionMode::Borrowed);
                    let value = extract_string_value(&nv)?;
                    if value.trim().is_empty() {
                        return Err(syn::Error::new_spanned(
                            &nv,
                            "borrowed_from cannot be empty",
                        ));
                    }
                    from_patterns.push(value);
                } else if nv.path.is_ident("borrowed_to") {
                    if to_pattern.is_some() {
                        return Err(syn::Error::new_spanned(
                            &nv,
                            "Duplicate 'borrowed_to' attribute: only one target type allowed per #[coerce(...)] attribute",
                        ));
                    }
                    mode = Some(CoercionMode::Borrowed);
                    to_mode_seen = Some(CoercionMode::Borrowed);
                    let value = extract_string_value(&nv)?;
                    if value.trim().is_empty() {
                        return Err(syn::Error::new_spanned(&nv, "borrowed_to cannot be empty"));
                    }
                    to_pattern = Some(value);
                } else if nv.path.is_ident("owned_from") {
                    mode = Some(CoercionMode::Owned);
                    from_mode_seen = Some(CoercionMode::Owned);
                    let value = extract_string_value(&nv)?;
                    if value.trim().is_empty() {
                        return Err(syn::Error::new_spanned(&nv, "owned_from cannot be empty"));
                    }
                    from_patterns.push(value);
                } else if nv.path.is_ident("owned_to") {
                    if to_pattern.is_some() {
                        return Err(syn::Error::new_spanned(
                            &nv,
                            "Duplicate 'owned_to' attribute: only one target type allowed per #[coerce(...)] attribute",
                        ));
                    }
                    mode = Some(CoercionMode::Owned);
                    to_mode_seen = Some(CoercionMode::Owned);
                    let value = extract_string_value(&nv)?;
                    if value.trim().is_empty() {
                        return Err(syn::Error::new_spanned(&nv, "owned_to cannot be empty"));
                    }
                    to_pattern = Some(value);
                } else if nv.path.is_ident("cloned_from") {
                    mode = Some(CoercionMode::Cloned);
                    from_mode_seen = Some(CoercionMode::Cloned);
                    let value = extract_string_value(&nv)?;
                    if value.trim().is_empty() {
                        return Err(syn::Error::new_spanned(&nv, "cloned_from cannot be empty"));
                    }
                    from_patterns.push(value);
                } else if nv.path.is_ident("cloned_to") {
                    if to_pattern.is_some() {
                        return Err(syn::Error::new_spanned(
                            &nv,
                            "Duplicate 'cloned_to' attribute: only one target type allowed per #[coerce(...)] attribute",
                        ));
                    }
                    mode = Some(CoercionMode::Cloned);
                    to_mode_seen = Some(CoercionMode::Cloned);
                    let value = extract_string_value(&nv)?;
                    if value.trim().is_empty() {
                        return Err(syn::Error::new_spanned(&nv, "cloned_to cannot be empty"));
                    }
                    to_pattern = Some(value);
                } else {
                    return Err(syn::Error::new_spanned(
                        &nv.path,
                        "Expected 'borrowed_from', 'borrowed_to', 'owned_from', 'owned_to', 'cloned_from', or 'cloned_to'",
                    ));
                }
            }
            syn::Meta::Path(path) => {
                if path.is_ident("asref") {
                    has_asref = true;
                } else {
                    return Err(syn::Error::new_spanned(
                        &path,
                        "Expected 'asref' marker (only valid for borrowed coercions)",
                    ));
                }
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    &meta,
                    "Expected name-value pair or path",
                ));
            }
        }
    }

    let mode = mode.ok_or_else(|| {
        syn::Error::new(
            attr.span(),
            "Missing coercion mode: use borrowed_from/to, owned_from/to, or cloned_from/to",
        )
    })?;

    if from_patterns.is_empty() {
        return Err(syn::Error::new(
            attr.span(),
            "Missing source types: at least one 'borrowed_from', 'owned_from', or 'cloned_from' required",
        ));
    }

    let to_pattern = to_pattern.ok_or_else(|| {
        syn::Error::new(
            attr.span(),
            "Missing target type: 'borrowed_to', 'owned_to', or 'cloned_to' required",
        )
    })?;

    // Validate that from_mode and to_mode match
    if let (Some(from_mode), Some(to_mode)) = (from_mode_seen, to_mode_seen) {
        if from_mode != to_mode {
            return Err(syn::Error::new(
                attr.span(),
                format!(
                    "Mismatched coercion modes: from side uses {:?} but to side uses {:?}. Both sides must use the same mode (e.g., borrowed_from with borrowed_to)",
                    from_mode, to_mode
                ),
            ));
        }
    }

    // Validate asref is only used with borrowed
    if has_asref && mode != CoercionMode::Borrowed {
        return Err(syn::Error::new(
            attr.span(),
            "asref marker is only valid for borrowed coercions",
        ));
    }

    // Check for no-op coercions (source == target)
    // This is a warning-level issue, but we'll make it an error for clarity
    for from_pattern in &from_patterns {
        if from_pattern.trim() == to_pattern.trim() {
            return Err(syn::Error::new(
                attr.span(),
                format!(
                    "No-op coercion detected: coercing from '{}' to '{}' (same type). This coercion has no effect and should be removed.",
                    from_pattern, to_pattern
                ),
            ));
        }
    }

    Ok(Some(CoercionSpec {
        from_patterns,
        to_pattern,
        kind: mode,
        generate_asref: has_asref,
    }))
}

fn extract_string_value(nv: &syn::MetaNameValue) -> syn::Result<String> {
    let syn::Expr::Lit(expr_lit) = &nv.value else {
        return Err(syn::Error::new_spanned(
            &nv.value,
            "Expected string literal",
        ));
    };

    let syn::Lit::Str(lit_str) = &expr_lit.lit else {
        return Err(syn::Error::new_spanned(
            &expr_lit.lit,
            "Expected string literal",
        ));
    };

    Ok(lit_str.value())
}

/// Split a string by `|` but only at top level (not inside angle brackets)
/// Example: "TypedPath<Absolute | Relative, _>" should be treated as one item (not split)
/// Example: "TypedPath<Absolute, _> | TypedPath<Relative, _>" splits into two items
/// But actually the INTENDED syntax is "Absolute | Relative" INSIDE the brackets
/// So "TypedPath<Absolute | Relative, _>" should split the type parameter position
fn split_by_pipe_respecting_brackets(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth = 0;

    for ch in s.chars() {
        match ch {
            '<' => {
                depth += 1;
                current.push(ch);
            }
            '>' => {
                depth -= 1;
                current.push(ch);
            }
            '|' if depth > 0 => {
                // Pipe inside brackets - this is for type alternatives
                // We need to expand this differently
                // For now, store the pipe
                current.push(ch);
            }
            '|' if depth == 0 => {
                // Top-level pipe, split here
                if !current.trim().is_empty() {
                    result.push(current.trim().to_string());
                    current.clear();
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }

    // If no top-level splits found but there are pipes inside brackets,
    // we need to expand those
    if result.len() <= 1 && s.contains('|') {
        // Parse and expand type parameter alternatives
        return expand_type_parameter_alternatives(s);
    }

    if result.is_empty() {
        vec![s.to_string()]
    } else {
        result
    }
}

/// Expand type parameter alternatives like "TypedPath<Absolute | Relative, _>"
/// into ["TypedPath<Absolute, _>", "TypedPath<Relative, _>"]
fn expand_type_parameter_alternatives(s: &str) -> Vec<String> {
    // Find the angle brackets
    let start = s.find('<');
    let end = s.rfind('>');

    if let (Some(start_pos), Some(end_pos)) = (start, end) {
        let prefix = &s[..start_pos + 1]; // "TypedPath<"
        let suffix = &s[end_pos..]; // ">"
        let params = &s[start_pos + 1..end_pos]; // "Absolute | Relative, _"

        // Split parameters by comma
        let param_parts: Vec<&str> = params.split(',').collect();

        // Find which parameter has | and expand it
        let mut expanded_params: Vec<Vec<String>> = Vec::new();

        for param in param_parts {
            if param.contains('|') {
                // This parameter has alternatives
                let alternatives: Vec<String> =
                    param.split('|').map(|s| s.trim().to_string()).collect();
                expanded_params.push(alternatives);
            } else {
                // Single value
                expanded_params.push(vec![param.trim().to_string()]);
            }
        }

        // Generate cartesian product
        let mut results = vec![String::new()];
        for alternatives in &expanded_params {
            let mut new_results = Vec::new();
            for result in &results {
                for alt in alternatives {
                    let mut new_result = result.clone();
                    if !new_result.is_empty() {
                        new_result.push_str(", ");
                    }
                    new_result.push_str(alt);
                    new_results.push(new_result);
                }
            }
            results = new_results;
        }

        // Combine prefix, params, and suffix
        return results
            .into_iter()
            .map(|params| format!("{}{}{}", prefix, params, suffix))
            .collect();
    }

    vec![s.to_string()]
}

/// Expand a CoercionSpec into concrete ParsedCoercion instances
/// Handles `|` syntax in from_patterns and generates cartesian product
fn expand_coercion_spec(
    spec: &CoercionSpec,
    generics: &syn::Generics,
) -> syn::Result<Vec<ParsedCoercion>> {
    // Split the to_pattern by | to get all target alternatives
    let to_alternatives = split_by_pipe_respecting_brackets(&spec.to_pattern);

    let mut result = Vec::new();

    // For each from_pattern, split by | and create separate coercions
    for from_pattern in &spec.from_patterns {
        // Split by | but only at the top level (not inside <>)
        let from_alternatives = split_by_pipe_respecting_brackets(from_pattern);

        for from_alternative in from_alternatives {
            let from_parsed = parse_target_with_type_holes(&from_alternative, generics)?;

            // For each to alternative, create a coercion (Cartesian product)
            for to_alternative in &to_alternatives {
                let to_parsed = parse_target_with_type_holes(to_alternative, generics)?;

                // Validate that type hole positions match between from and to
                if from_parsed.type_hole_positions != to_parsed.type_hole_positions {
                    return Err(syn::Error::new(
                        proc_macro2::Span::call_site(),
                        format!(
                            "Type hole positions mismatch: from pattern '{}' has type holes at {:?}, but to pattern '{}' has type holes at {:?}",
                            from_alternative,
                            from_parsed.type_hole_positions,
                            to_alternative,
                            to_parsed.type_hole_positions
                        ),
                    ));
                }

                result.push(ParsedCoercion {
                    source_type: from_parsed.target_type.clone(),
                    target_type: to_parsed.target_type.clone(),
                    type_hole_positions: from_parsed.type_hole_positions.clone(),
                });
            }
        }
    }

    Ok(result)
}

/// Extract only the generic parameters at type hole positions
/// Returns a TokenStream like `<Type>` or `<Base, Type>` or ``
fn extract_type_hole_generics(
    generics: &syn::Generics,
    type_hole_positions: &[usize],
) -> proc_macro2::TokenStream {
    if type_hole_positions.is_empty() {
        // No type holes means fully concrete types, no generics needed
        return quote! {};
    }

    let type_params: Vec<&Ident> = generics
        .params
        .iter()
        .filter_map(|p| {
            if let syn::GenericParam::Type(tp) = p {
                Some(&tp.ident)
            } else {
                None
            }
        })
        .collect();

    let type_hole_params: Vec<_> = type_hole_positions
        .iter()
        .filter_map(|&pos| type_params.get(pos).copied())
        .collect();

    if type_hole_params.is_empty() {
        quote! {}
    } else {
        quote! { <#(#type_hole_params),*> }
    }
}

fn generate_borrowed_impl(
    struct_name: &Ident,
    generics: &syn::Generics,
    trait_name: &Ident,
    coercion: &ParsedCoercion,
    fields: &syn::FieldsNamed,
    _phantom_fields: &[&Ident],
) -> syn::Result<proc_macro2::TokenStream> {
    let source_type = &coercion.source_type;
    let target_type = &coercion.target_type;

    let Type::Path(target_path) = target_type else {
        return Err(syn::Error::new_spanned(
            target_type,
            "Coerce target must be a type path",
        ));
    };

    let target_segment = target_path.path.segments.last().unwrap();
    let PathArguments::AngleBracketed(_target_args) = &target_segment.arguments else {
        return Err(syn::Error::new_spanned(
            target_type,
            "Coerce target must have type parameters",
        ));
    };

    // Generate destructuring pattern with type annotations for all fields
    let field_destructure: Vec<_> = fields
        .named
        .iter()
        .map(|f| {
            let field_name = f.ident.as_ref().unwrap();
            quote! { #field_name: _ }
        })
        .collect();

    // Extract only the generic parameters that appear in type holes
    // For the impl, we need generics only for the type hole positions
    let generics_for_impl = extract_type_hole_generics(generics, &coercion.type_hole_positions);

    Ok(quote! {
        impl #generics_for_impl #trait_name<#target_type> for #source_type {
            fn coerce(&self) -> &#target_type {
                // Compile-time safety guards: ensure all fields are accounted for
                let #struct_name { #(#field_destructure),* } = self;

                // SAFETY: Types differ only in PhantomData type parameters.
                // The destructuring pattern above ensures this at compile time.
                unsafe { std::mem::transmute(self) }
            }
        }
    })
}

fn generate_owned_impl(
    struct_name: &Ident,
    generics: &syn::Generics,
    trait_name: &Ident,
    coercion: &ParsedCoercion,
    fields: &syn::FieldsNamed,
    _phantom_fields: &[&Ident],
) -> syn::Result<proc_macro2::TokenStream> {
    let source_type = &coercion.source_type;
    let target_type = &coercion.target_type;

    let Type::Path(target_path) = target_type else {
        return Err(syn::Error::new_spanned(
            target_type,
            "Coerce target must be a type path",
        ));
    };

    let target_segment = target_path.path.segments.last().unwrap();
    let PathArguments::AngleBracketed(_target_args) = &target_segment.arguments else {
        return Err(syn::Error::new_spanned(
            target_type,
            "Coerce target must have type parameters",
        ));
    };

    // Generate destructuring pattern for all fields
    let field_destructure: Vec<_> = fields
        .named
        .iter()
        .map(|f| {
            let field_name = f.ident.as_ref().unwrap();
            quote! { #field_name: _ }
        })
        .collect();

    let generics_for_impl = extract_type_hole_generics(generics, &coercion.type_hole_positions);

    Ok(quote! {
        impl #generics_for_impl #trait_name<#target_type> for #source_type {
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
    coercion: &ParsedCoercion,
    fields: &syn::FieldsNamed,
    _phantom_fields: &[&Ident],
) -> syn::Result<proc_macro2::TokenStream> {
    let source_type = &coercion.source_type;
    let target_type = &coercion.target_type;

    let Type::Path(target_path) = target_type else {
        return Err(syn::Error::new_spanned(
            target_type,
            "Coerce target must be a type path",
        ));
    };

    let target_segment = target_path.path.segments.last().unwrap();
    let PathArguments::AngleBracketed(_target_args) = &target_segment.arguments else {
        return Err(syn::Error::new_spanned(
            target_type,
            "Coerce target must have type parameters",
        ));
    };

    // Generate destructuring pattern for all fields
    let field_destructure: Vec<_> = fields
        .named
        .iter()
        .map(|f| {
            let field_name = f.ident.as_ref().unwrap();
            quote! { #field_name: _ }
        })
        .collect();

    let generics_for_impl = extract_type_hole_generics(generics, &coercion.type_hole_positions);

    // Build where clause with Clone bound on the source type
    let where_clause = quote! { where #source_type: Clone };

    Ok(quote! {
        impl #generics_for_impl #trait_name<#target_type> for #source_type #where_clause {
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
    _struct_name: &Ident,
    generics: &syn::Generics,
    _trait_name: &Ident,
    coercion: &ParsedCoercion,
) -> syn::Result<proc_macro2::TokenStream> {
    let source_type = &coercion.source_type;
    let target_type = &coercion.target_type;
    let generics_for_impl = extract_type_hole_generics(generics, &coercion.type_hole_positions);

    Ok(quote! {
        impl #generics_for_impl AsRef<#target_type> for #source_type {
            fn as_ref(&self) -> &#target_type {
                self.coerce()
            }
        }
    })
}
