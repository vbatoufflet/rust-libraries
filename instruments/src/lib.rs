use std::{env, result, str::FromStr, vec::Vec};

use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions as semconv;
use serde::Deserialize;
use tracing_subscriber::{prelude::*, EnvFilter};

use config::prelude::*;
use errors::prelude::*;

use crate::layers::{logs, metrics, traces};

#[cfg(feature = "rpc")]
use crate::layers::rpc::RPCLayer;

const SCOPE_NAME: &str = "rust-libraries/instruments";

pub mod prelude {
    pub use paste::paste as __internal_paste;
    pub use tracing;
    pub use tracing::{instrument, Level};

    pub use crate::{counter, histogram};
    pub use crate::{debug, error, info, trace, warn};
}

mod layers;

#[cfg(feature = "rpc")]
pub mod rpc;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Configuration(String),

    #[error("{0}")]
    Internal(String),
}

#[derive(Debug, Eq, PartialEq)]
pub enum Exporter {
    Console,
    None,
    Otlp,
}

impl FromStr for Exporter {
    type Err = Error;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "console" => Ok(Self::Console),
            "none" => Ok(Self::None),
            "otlp" => Ok(Self::Otlp),
            _ => Err(Error::Configuration(format!("unsupported exporter: {s}"))),
        }
    }
}

pub fn exporters_from_env(key: &str) -> Result<Vec<Exporter>, Error> {
    env::var(key).map_or_else(
        |_| Ok(vec![Exporter::Console]),
        |s| s.split(',').map(|s| s.trim().parse()).collect(),
    )
}

#[derive(Config, Debug, Deserialize)]
pub struct Config {
    #[config(default = "info")]
    pub logs_filter: String,

    #[config(default = "info")]
    pub metrics_filter: String,

    #[config(default = "info")]
    pub traces_filter: String,
}

pub fn new(service_name: &'static str, service_version: &'static str) -> Result<(), Error> {
    let config = Config::from_env("INSTRUMENTS").map_err(|v| Error::Configuration(v.to_string()))?;

    let mut pairs = vec![
        KeyValue::new(semconv::resource::OTEL_SCOPE_NAME, SCOPE_NAME),
        KeyValue::new(semconv::resource::OTEL_SCOPE_VERSION, env!("CARGO_PKG_VERSION")),
        KeyValue::new(semconv::resource::SERVICE_NAME, service_name.to_string()),
        KeyValue::new(semconv::resource::SERVICE_VERSION, service_version.to_string()),
    ];
    if let Ok(env) = env::var("ENV") {
        pairs.push(KeyValue::new(semconv::resource::DEPLOYMENT_ENVIRONMENT_NAME, env));
    }

    let resource = Resource::builder().with_attributes(pairs).build();

    let logs_layer = logs::new_layer(resource.clone())?;
    let metrics_layer = metrics::new_layer(resource.clone())?;
    let traces_layer = traces::new_layer(service_name, resource)?;

    let registry = tracing_subscriber::registry();

    #[cfg(feature = "rpc")]
    let registry = registry.with(RPCLayer);

    registry
        .with(logs_layer.with_filter(EnvFilter::new(&config.logs_filter)))
        .with(metrics_layer.with_filter(EnvFilter::new(&config.metrics_filter)))
        .with(traces_layer.with_filter(EnvFilter::new(&config.traces_filter)))
        .init();

    Ok(())
}
