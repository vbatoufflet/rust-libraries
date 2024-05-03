use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::{
    logs::{Config, LoggerProvider},
    runtime, Resource,
};

use errors::prelude::*;

use crate::{Error, Exporter};

use super::console;

pub(crate) fn new_layer(
    resource: Resource,
    exporter: Exporter,
) -> Result<OpenTelemetryTracingBridge<LoggerProvider, opentelemetry_sdk::logs::Logger>, Error> {
    let logger_provider = new_provider(resource, exporter)?;

    Ok(OpenTelemetryTracingBridge::new(&logger_provider))
}

fn new_provider(resource: Resource, exporter: Exporter) -> Result<LoggerProvider, Error> {
    let config = Config::default().with_resource(resource);

    let provider = match exporter {
        Exporter::Console => LoggerProvider::builder()
            .with_config(config)
            .with_simple_exporter(console::logs::LogsExporter::default())
            .build(),

        Exporter::Noop => LoggerProvider::builder().with_config(config).build(),

        Exporter::Otlp => {
            let exporter = opentelemetry_otlp::new_exporter()
                .tonic()
                .build_log_exporter()
                .map_err(|v| Error::Internal(v.to_string()))?;

            LoggerProvider::builder()
                .with_config(config)
                .with_batch_exporter(exporter, runtime::Tokio)
                .build()
        }

        Exporter::Stdout => {
            let exporter = opentelemetry_stdout::LogExporterBuilder::default().build();

            LoggerProvider::builder()
                .with_config(config)
                .with_simple_exporter(exporter)
                .build()
        }
    };

    Ok(provider)
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
