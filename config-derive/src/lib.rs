use proc_macro::TokenStream;

use darling::FromField;
use quote::quote;
use syn::{Data, DataStruct, Fields, Ident};

#[proc_macro_derive(Config, attributes(config))]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    expand_derive_config(&ast)
}

fn expand_derive_config(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let fields = match &ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,

        _ => panic!("expected a struct with named fields"),
    };

    let (field_name, field_default): (Vec<&Ident>, Vec<String>) = fields
        .iter()
        .filter_map(|field| match DeriveArgs::from_field(field).ok()?.default? {
            default if !default.is_empty() => Some((field.ident.as_ref()?, default)),
            _ => None,
        })
        .unzip();

    quote! {
        impl ConfigTrait for #name {
            fn from_env(prefix: &str) -> Result<Self, ConfigError> {
                config::__internal::Config::builder()
                    #(
                    .set_default(stringify!(#field_name), #field_default)?
                    )*
                    .add_source(config::__internal::Environment::with_prefix(prefix))
                    .build()?
                    .try_deserialize()
            }
        }
    }
    .into()
}

#[derive(Default, FromField)]
#[darling(default, attributes(config), forward_attrs(allow, doc, cfg))]
struct DeriveArgs {
    default: Option<String>,
}
