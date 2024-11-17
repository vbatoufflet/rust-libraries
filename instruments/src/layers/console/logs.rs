use std::{fmt::Debug, fmt::Formatter, io};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use colored::Colorize;
use opentelemetry::logs::AnyValue;
use opentelemetry_sdk::export::logs::{ExportResult, LogBatch};

use super::severity_to_str;

pub struct LogExporter {
    writer: Option<Box<dyn io::Write + Send + Sync>>,
}

impl Default for LogExporter {
    fn default() -> Self {
        Self {
            writer: Some(Box::new(io::stdout())),
        }
    }
}

impl Debug for LogExporter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("LogsExporter")
    }
}

#[async_trait]
impl opentelemetry_sdk::export::logs::LogExporter for LogExporter {
    async fn export(&mut self, batch: LogBatch<'_>) -> ExportResult {
        let Some(writer) = &mut self.writer else {
            return Err("exporter is shut down".into());
        };

        for (record, _) in batch.iter() {
            let ts = match record.observed_timestamp.or(record.timestamp) {
                Some(v) => Into::<DateTime<Utc>>::into(v),
                None => continue,
            };

            let severity = severity_to_str(record.severity_number);

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

            let body = if let Some(AnyValue::String(body)) = &record.body {
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
                let _ = writer.write_fmt(format_args!("{} {}", ",".dimmed(), attributes.join(" "),));
            }

            let _ = writer.write(b"\n");
        }

        Ok(())
    }

    fn shutdown(&mut self) {
        self.writer.take();
    }
}
