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

    let mut list_fields: Vec<&Ident> = vec![];
    let mut skip_prefix_overrides: Vec<proc_macro2::TokenStream> = vec![];

    let (field_name, field_default): (Vec<&Ident>, Vec<proc_macro2::TokenStream>) = fields
        .iter()
        .filter_map(|field| {
            let field_name = field.ident.as_ref()?;
            let args = DeriveArgs::from_field(field).ok()?;

            let is_list = args.list.unwrap_or(false);
            if is_list {
                list_fields.push(field_name);
            }

            if args.skip_prefix.unwrap_or(false) {
                let override_value = if is_list {
                    quote! {
                        std::env::var(stringify!(#field_name).to_uppercase())
                            .ok()
                            .map(|v| v.split(',').map(|s| s.to_string()).collect::<Vec<_>>())
                    }
                } else {
                    quote! {
                        std::env::var(stringify!(#field_name).to_uppercase()).ok()
                    }
                };

                skip_prefix_overrides.push(quote! {
                    .set_override_option(stringify!(#field_name), #override_value)?
                });
            }

            let field_default = match args.default? {
                DefaultValue::Int(int) => quote! { #int },
                DefaultValue::Path(path) if is_list => quote! { #path.to_vec() },
                DefaultValue::Path(path) => quote! { #path },
                DefaultValue::Str(string) => quote! { #string },
            };

            Some((field_name, field_default))
        })
        .unzip();

    let list_config = if list_fields.is_empty() {
        quote! {}
    } else {
        quote! {
            .try_parsing(true)
            .list_separator(",")
            #(
            .with_list_parse_key(stringify!(#list_fields))
            )*
        }
    };

    quote! {
        impl ConfigTrait for #name {
            fn from_env(prefix: &str) -> Result<Self, ConfigError> {
                config::__internal::Config::builder()
                    #(
                    .set_default(stringify!(#field_name), #field_default)?
                    )*
                    #(
                    #skip_prefix_overrides
                    )*
                    .add_source(
                        config::__internal::Environment::with_prefix(prefix)
                            #list_config
                    )
                    .build()?
                    .try_deserialize()
                    .map_err(|err| {
                        if let ConfigError::NotFound(key) = &err {
                            return ConfigError::NotFound(format!("{}_{}", prefix, key.to_uppercase()));
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
    list: Option<bool>,
    skip_prefix: Option<bool>,
}

#[derive(Debug)]
enum DefaultValue {
    Int(i64),
    Path(ExprPath),
    Str(String),
}

impl darling::FromMeta for DefaultValue {
    fn from_meta(meta: &Meta) -> darling::Result<Self> {
        match meta {
            Meta::NameValue(name_value) => match &name_value.value {
                Expr::Lit(syn::ExprLit {
                    lit: Lit::Int(lit_int),
                    ..
                }) => lit_int
                    .base10_parse()
                    .map(Self::Int)
                    .map_err(|err| darling::Error::custom(err.to_string())),

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
