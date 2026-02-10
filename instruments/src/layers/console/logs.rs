use std::{
    fmt::{self, Debug, Formatter, Write as _},
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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("LogsExporter")
    }
}

impl opentelemetry_sdk::logs::LogExporter for LogExporter {
    async fn export(&self, batch: LogBatch<'_>) -> OTelSdkResult {
        let Ok(writer) = &mut self.writer.lock() else {
            return Err(OTelSdkError::AlreadyShutdown);
        };

        let mut attr_buf = String::new();

        for (record, _) in batch.iter() {
            let ts = match record.observed_timestamp().or_else(|| record.timestamp()) {
                Some(v) => Into::<DateTime<Utc>>::into(v),
                None => continue,
            };

            let severity = severity_to_str(record.severity_number());

            let body = match record.body() {
                Some(AnyValue::String(body)) => body.as_str(),
                _ => continue,
            };

            attr_buf.clear();
            for (key, value) in record.attributes_iter() {
                let v: &dyn std::fmt::Display = match value {
                    AnyValue::Int(v) => v,
                    AnyValue::Double(v) => v,
                    AnyValue::String(v) => v,
                    AnyValue::Boolean(v) => v,
                    _ => continue,
                };
                let _ = write!(attr_buf, " {}{v}", format!("{}=", key.as_str()).dimmed());
            }

            let _ = writer.write_fmt(format_args!(
                "{} {:>7} {}",
                ts.to_rfc3339_opts(chrono::SecondsFormat::Nanos, false).dimmed(),
                severity,
                body,
            ));

            if !attr_buf.is_empty() {
                if !body.is_empty() {
                    let _ = writer.write_fmt(format_args!("{}", ",".dimmed()));
                }
                let _ = writer.write_fmt(format_args!("{attr_buf}"));
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
