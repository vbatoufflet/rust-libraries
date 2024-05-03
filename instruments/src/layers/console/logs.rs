use std::{fmt::Debug, fmt::Formatter, io};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use colored::Colorize;
use opentelemetry::logs::AnyValue;
use opentelemetry_sdk::export::logs::{ExportResult, LogData};

use super::severity_to_str;

pub struct LogsExporter {
    writer: Option<Box<dyn io::Write + Send + Sync>>,
}

impl Default for LogsExporter {
    fn default() -> Self {
        LogsExporter {
            writer: Some(Box::new(io::stdout())),
        }
    }
}

impl Debug for LogsExporter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("LogsExporter")
    }
}

#[async_trait]
impl opentelemetry_sdk::export::logs::LogExporter for LogsExporter {
    async fn export(&mut self, batch: Vec<LogData>) -> ExportResult {
        if let Some(writer) = &mut self.writer {
            for log_data in batch {
                let ts = match log_data.record.observed_timestamp.or(log_data.record.timestamp) {
                    Some(v) => Into::<DateTime<Utc>>::into(v),
                    None => continue,
                };

                let severity = severity_to_str(log_data.record.severity_number);

                let attributes: Vec<String> = log_data
                    .record
                    .attributes
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|(key, value)| {
                        let mut key: String = key.into();
                        key.push_str("=");

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

                let body = if let Some(AnyValue::String(body)) = log_data.record.body {
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

                if attributes.len() > 0 {
                    let _ = writer.write_fmt(format_args!("{} {}", ",".dimmed(), attributes.join(" "),));
                }

                let _ = writer.write(b"\n");
            }
            Ok(())
        } else {
            Err("exporter is shut down".into())
        }
    }

    fn shutdown(&mut self) {
        self.writer.take();
    }
}
