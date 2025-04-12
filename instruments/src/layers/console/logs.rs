use std::{
    fmt::{Debug, Formatter},
    io::{self, Stdout, Write},
    sync::Mutex,
};

use chrono::{DateTime, Utc};
use colored::Colorize;
use opentelemetry::logs::AnyValue;
use opentelemetry_sdk::{
    error::{OTelSdkError, OTelSdkResult},
    logs::LogBatch,
};

use super::severity_to_str;

pub struct LogExporter {
    writer: Mutex<Stdout>,
}

impl Default for LogExporter {
    fn default() -> Self {
        Self {
            writer: Mutex::new(io::stdout()),
        }
    }
}

impl Debug for LogExporter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("LogsExporter")
    }
}

impl opentelemetry_sdk::logs::LogExporter for LogExporter {
    async fn export(&self, batch: LogBatch<'_>) -> OTelSdkResult {
        let Ok(writer) = &mut self.writer.lock() else {
            return Err(OTelSdkError::AlreadyShutdown);
        };

        for (record, _) in batch.iter() {
            let ts = match record.observed_timestamp().or_else(|| record.timestamp()) {
                Some(v) => Into::<DateTime<Utc>>::into(v),
                None => continue,
            };

            let severity = severity_to_str(record.severity_number());

            let attributes: Vec<String> = record
                .attributes_iter()
                .filter_map(|(key, value)| {
                    let mut key: String = key.as_str().to_string();
                    key.push('=');

                    let value = match value {
                        AnyValue::Int(value) => value.to_string(),
                        AnyValue::Double(value) => value.to_string(),
                        AnyValue::String(value) => value.to_string(),
                        AnyValue::Boolean(value) => value.to_string(),
                        _ => return None,
                    };

                    Some(format!("{}{}", key.dimmed(), value))
                })
                .collect();

            let body = if let Some(AnyValue::String(body)) = &record.body() {
                body.to_string()
            } else {
                continue;
            };

            let _ = writer.write_fmt(format_args!(
                "{} {:>7} {}",
                ts.to_rfc3339_opts(chrono::SecondsFormat::Nanos, false).dimmed(),
                severity,
                body,
            ));

            if !attributes.is_empty() {
                if !body.is_empty() {
                    let _ = writer.write_fmt(format_args!("{}", ", ".dimmed()));
                }
                let _ = writer.write_fmt(format_args!("{}", attributes.join(" ")));
            }

            let _ = writer.write(b"\n");
        }

        Ok(())
    }

    fn shutdown(&self) -> OTelSdkResult {
        self.writer
            .lock()
            .map_err(|err| OTelSdkError::InternalFailure(err.to_string()))?
            .flush()
            .map_err(|err| OTelSdkError::InternalFailure(err.to_string()))
    }
}
