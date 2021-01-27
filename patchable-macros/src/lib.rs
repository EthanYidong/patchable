//! Derive macro(s) for patchable structs in Rust.
//! You probably want [`patchable`](https://docs.rs/patchable) instead.

use proc_macro::TokenStream;
use syn::*;
use quote::*;

/// Creates a patch struct and implements [`Patchable`](trait@patchable_core::Patchable).
/// 
/// ```
/// #[derive(Patchable)]
/// struct MyStruct {
///     foo: String,
///     bar: i32
/// }
/// ```
/// would generate 
/// ```
/// pub struct MyStructPatch {
///     pub foo: Option<String>,
///     pub bar: Option<String>
/// }
/// 
/// impl Patchable<MyStructPatch> for MyStruct {
///     // skipped for brevity
/// }
/// ```
/// 
/// You can also specify using the `#[patch(PatchType)]` attribute to change both the name of the generated struct,
/// and the type of the replacement field in the generate struct. This works as long as the field type implements `Patchable<PatchType>`.
/// 
/// This allows for nesting of patches.
/// 
/// ```
/// #[derive(Patchable)]
/// #[patch(MyPatch)]
/// struct MyStruct {
///     foo: String,
///     #[patch(BarPatch)]
///     bar: Bar,
/// }
/// 
/// #[derive(Patchable)]
/// struct Bar {
///     foobar: i32,
/// }
/// ```
/// would generate
/// ```
/// struct MyPatch {
///     foo: Option<String>,
///     bar: BarPatch,
/// }
/// 
/// struct BarPatch {
///     foobar: Option<i32>,
/// }
/// // Patchable impls...
/// ```
#[proc_macro_derive(Patchable, attributes(patch))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input_name = input.ident;

    let patch_name = if let Some(name) = parse_attrs_ident(input.attrs) {
        name
    } else {
        format_ident!("{}Patch", input_name)
    };

    let mut names = Vec::new();
    let mut types = Vec::new();
    let mut patch_types = Vec::new();

    match input.data {
        Data::Struct(struct_data) => {
            match struct_data.fields {
                Fields::Named(named_fields) => {
                    for field in named_fields.named {
                        let ty = field.ty;
                        names.push(field.ident.unwrap());
                        types.push(quote!{#ty});
                        if let Some(ident) = parse_attrs_ident(field.attrs) {
                            patch_types.push(quote!{#ident});
                        } else {
                            patch_types.push(quote!{::core::option::Option<#ty>});
                        }
                    }
                },
                Fields::Unnamed(_unnamed_fields) => unimplemented!(),
                Fields::Unit => unimplemented!(),
            }
        },
        Data::Enum(_enum_data) => unimplemented!(),
        Data::Union(_union_data) => unimplemented!(),
    }

    let patch_struct = quote!{
        pub struct #patch_name {
            #(pub #names: #patch_types),*
        }
    };

    let patchable_impl = quote!{
        impl ::patchable_core::Patchable<#patch_name> for #input_name {
            fn apply_patch(&mut self, patch: #patch_name) {
                #(
                    self.#names.apply_patch(patch.#names);
                )*
            }
        }
    };

    let output = quote!{
        #patch_struct
        #patchable_impl
    };

    TokenStream::from(output)
}

fn parse_attrs_ident(attrs: Vec<Attribute>) -> Option<Ident> {
    for attr in attrs {
        if let Some(ident) = parse_attr_ident(attr) {
            return Some(ident);
        }
    }
    None
}

fn parse_attr_ident(attr: Attribute) -> Option<Ident> {
    if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
        if meta_list.path.is_ident("patch") {
            if let Some(NestedMeta::Meta(Meta::Path(path))) = meta_list.nested.first() {
                return path.get_ident().cloned();
            }
        }
    }
    None
}
