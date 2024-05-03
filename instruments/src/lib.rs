use std::{env, result, str::FromStr};

use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions as semconv;
use serde::Deserialize;
use tracing_subscriber::{prelude::*, EnvFilter};

use config::prelude::*;
use errors::prelude::*;

use crate::layers::{logs, metrics, rpc::RPCLayer, traces};

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

#[derive(Debug, PartialEq)]
pub enum Exporter {
    Console,
    Noop,
    Otlp,
    Stdout,
}

impl FromStr for Exporter {
    type Err = Error;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "console" => Ok(Exporter::Console),
            "noop" => Ok(Exporter::Noop),
            "otlp" => Ok(Exporter::Otlp),
            "stdout" => Ok(Exporter::Stdout),
            _ => Err(Error::Configuration(format!("unsupported exporter: {}", s))),
        }
    }
}

#[derive(Config, Debug, Deserialize)]
pub struct Config {
    #[config(default = "console")]
    pub logs_exporter: String,

    #[config(default = "info")]
    pub logs_filter: String,

    #[config(default = "noop")]
    pub metrics_exporter: String,

    #[config(default = "info")]
    pub metrics_filter: String,

    #[config(default = "noop")]
    pub traces_exporter: String,

    #[config(default = "info")]
    pub traces_filter: String,

    #[config(default = "0")]
    pub traces_ratio_sample: f64,
}

pub fn new(service_name: &'static str, service_version: &'static str) -> Result<(), Error> {
    let config = Config::from_env("INSTRUMENTS").map_err(|v| Error::Configuration(v.to_string()))?;

    let logs_exporter = config.logs_exporter.parse()?;
    let metrics_exporter = config.metrics_exporter.parse()?;
    let traces_exporter = config.traces_exporter.parse()?;

    let mut pairs = vec![
        KeyValue::new(semconv::resource::OTEL_SCOPE_NAME, SCOPE_NAME),
        KeyValue::new(semconv::resource::OTEL_SCOPE_VERSION, env!("CARGO_PKG_VERSION")),
        KeyValue::new(semconv::resource::SERVICE_NAME, service_name.to_string()),
        KeyValue::new(semconv::resource::SERVICE_VERSION, service_version.to_string()),
    ];
    if let Ok(env) = env::var("ENV") {
        pairs.push(KeyValue::new(semconv::resource::DEPLOYMENT_ENVIRONMENT, env));
    }

    let resource = Resource::new(pairs);

    let logs_layer = logs::new_layer(resource.clone(), logs_exporter)?;

    let metrics_layer = metrics::new_layer(resource.clone(), metrics_exporter)?;

    let traces_layer = traces::new_layer(
        &service_name,
        resource,
        traces_exporter,
        config.traces_ratio_sample,
    )?;

    tracing_subscriber::registry()
        .with(RPCLayer)
        .with(logs_layer.with_filter(EnvFilter::new(&config.logs_filter)))
        .with(metrics_layer.with_filter(EnvFilter::new(&config.metrics_filter)))
        .with(traces_layer.with_filter(EnvFilter::new(&config.traces_filter)))
        .init();

    Ok(())
}
