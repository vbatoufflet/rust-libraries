use darling::FromField;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, Expr, ExprPath, Fields, Ident, Lit, Meta};

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

    let (field_name, field_default): (Vec<&Ident>, Vec<proc_macro2::TokenStream>) = fields
        .iter()
        .filter_map(|field| {
            let field_name = field.ident.as_ref()?;
            let field_default = match DeriveArgs::from_field(field).ok()?.default? {
                DefaultValue::Path(path) => quote! { #path },
                DefaultValue::Str(string) => quote! { #string },
            };

            Some((field_name, field_default))
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
                    .map_err(|err| {
                        let msg = err.to_string();
                        if msg.starts_with("missing field `") && msg.ends_with("`") {
                            if let Some(field_name) = msg.split('`').nth(1) {
                                return ConfigError::Message(
                                    format!(
                                        r#"environment variable "{}_{}" is not set"#,
                                        prefix,
                                        field_name.to_uppercase(),
                                    )
                                );
                            }
                        }
                        err
                    })
            }
        }
    }
    .into()
}

#[derive(Debug, Default, FromField)]
#[darling(default, attributes(config), forward_attrs(allow, doc, cfg))]
struct DeriveArgs {
    default: Option<DefaultValue>,
}

#[derive(Debug)]
enum DefaultValue {
    Path(ExprPath),
    Str(String),
}

impl darling::FromMeta for DefaultValue {
    fn from_meta(meta: &Meta) -> darling::Result<Self> {
        match meta {
            Meta::NameValue(name_value) => match &name_value.value {
                Expr::Lit(syn::ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) => Ok(Self::Str(lit_str.value())),

                Expr::Path(path) => Ok(Self::Path(path.clone())),

                _ => Err(darling::Error::unsupported_format("unsupported default value")),
            },

            _ => Err(darling::Error::unsupported_format("unsupported meta")),
        }
    }
}
