use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::{logs::LoggerProvider, runtime, Resource};

use errors::prelude::*;

use crate::{Error, Exporter};

use super::console;

pub fn new_layer(
    resource: Resource,
    exporter: &Exporter,
) -> Result<OpenTelemetryTracingBridge<LoggerProvider, opentelemetry_sdk::logs::Logger>, Error> {
    let logger_provider = new_provider(resource, exporter)?;

    Ok(OpenTelemetryTracingBridge::new(&logger_provider))
}

fn new_provider(resource: Resource, exporter: &Exporter) -> Result<LoggerProvider, Error> {
    let provider = match exporter {
        Exporter::Console => LoggerProvider::builder()
            .with_resource(resource)
            .with_simple_exporter(console::logs::LogExporter::default())
            .build(),

        Exporter::Noop => LoggerProvider::builder().with_resource(resource).build(),

        Exporter::Otlp => {
            let exporter = opentelemetry_otlp::LogExporter::builder()
                .with_tonic()
                .build()
                .map_err(|v| Error::Internal(v.to_string()))?;

            LoggerProvider::builder()
                .with_resource(resource)
                .with_batch_exporter(exporter, runtime::Tokio)
                .build()
        }

        Exporter::Stdout => {
            let exporter = opentelemetry_stdout::LogExporter::default();

            LoggerProvider::builder()
                .with_resource(resource)
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
