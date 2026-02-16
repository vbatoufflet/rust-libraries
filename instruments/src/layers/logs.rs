use std::env;

use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::{
    logs::{SdkLogger, SdkLoggerProvider},
    Resource,
};

use errors::prelude::*;

use crate::{exporters_from_env, Error, Exporter, LOGGER_PROVIDER};

use super::console;

pub fn new_layer(
    resource: Resource,
) -> Result<OpenTelemetryTracingBridge<SdkLoggerProvider, SdkLogger>, Error> {
    let logger_provider = new_provider(resource)?;
    let _ = LOGGER_PROVIDER.set(logger_provider.clone());
    Ok(OpenTelemetryTracingBridge::new(&logger_provider))
}

fn new_provider(resource: Resource) -> Result<SdkLoggerProvider, Error> {
    let mut builder = SdkLoggerProvider::builder().with_resource(resource);

    for exporter in exporters_from_env("OTEL_LOGS_EXPORTER")? {
        match exporter {
            #[cfg(feature = "stdout")]
            Exporter::Console => match env::var("OTEL_LOGS_EXPORTER_CONSOLE") {
                Ok(value) if value == "pretty" => {
                    let exporter = console::logs::LogExporter::default();
                    builder = builder.with_batch_exporter(exporter);
                }
                _ => {
                    let exporter = opentelemetry_stdout::LogExporter::default();
                    builder = builder.with_batch_exporter(exporter);
                }
            },

            Exporter::None => {
                // No-op
            }

            #[cfg(feature = "otlp")]
            Exporter::Otlp => {
                let exporter = opentelemetry_otlp::LogExporter::builder()
                    .with_tonic()
                    .build()
                    .map_err(|v| Error::Internal(v.to_string()))?;

                builder = builder.with_batch_exporter(exporter);
            }

            #[cfg(not(all(feature = "otlp", feature = "stdout")))]
            _ => Err(Error::Configuration(format!(
                "unsupported logs exporter: {exporter:?}",
            )))?,
        }
    }

    Ok(builder.build())
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => {
        tracing::trace!(
            log.target = std::module_path!(), log.file = std::file!(), log.line = std::line!(),
            $($arg)+
        )
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        tracing::debug!(
            log.target = std::module_path!(), log.file = std::file!(), log.line = std::line!(),
            $($arg)+
        )
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        tracing::info!(
            log.target = std::module_path!(), log.file = std::file!(), log.line = std::line!(),
            $($arg)+
        )
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {
        tracing::warn!(
            log.target = std::module_path!(), log.file = std::file!(), log.line = std::line!(),
            $($arg)+
        )
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        tracing::error!(
            log.target = std::module_path!(), log.file = std::file!(), log.line = std::line!(),
            $($arg)+
        )
    };
}
