#[cfg(feature = "otlp")]
use std::env;

use opentelemetry::global;
use opentelemetry_sdk::{
    metrics::{PeriodicReader, SdkMeterProvider},
    Resource,
};
use tracing::Subscriber;
use tracing_opentelemetry::MetricsLayer;
use tracing_subscriber::registry::LookupSpan;

#[cfg(feature = "otlp")]
use opentelemetry_sdk::metrics::Temporality;

use crate::{exporters_from_env, Error, Exporter};

pub fn new_layer<S>(resource: Resource) -> Result<MetricsLayer<S, SdkMeterProvider>, Error>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let meter_provider = new_provider(resource)?;
    global::set_meter_provider(meter_provider.clone());
    Ok(MetricsLayer::new(meter_provider))
}

fn new_provider(resource: Resource) -> Result<SdkMeterProvider, Error> {
    let mut builder = SdkMeterProvider::builder().with_resource(resource);

    for exporter in exporters_from_env("OTEL_METRICS_EXPORTER")? {
        match exporter {
            #[cfg(feature = "stdout")]
            Exporter::Console => {
                let exporter = opentelemetry_stdout::MetricExporter::builder().build();
                builder = builder.with_reader(PeriodicReader::builder(exporter).build());
            }

            Exporter::None => {
                // No-op
            }

            #[cfg(feature = "otlp")]
            Exporter::Otlp => {
                let temporality = match env::var("OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE")
                    .unwrap_or_else(|_| "cumulative".to_owned())
                    .to_ascii_lowercase()
                    .as_str()
                {
                    "cumulative" => Temporality::Cumulative,
                    "delta" => Temporality::Delta,
                    "lowmemory" => Temporality::LowMemory,
                    s => {
                        return Err(Error::Configuration(format!(
                            "unsupported metrics temporality: {s}",
                        )))
                    }
                };

                let exporter = opentelemetry_otlp::MetricExporter::builder()
                    .with_tonic()
                    .with_temporality(temporality)
                    .build()
                    .map_err(|v| Error::Internal(v.to_string()))?;

                builder = builder.with_reader(PeriodicReader::builder(exporter).build());
            }

            #[cfg(not(all(feature = "otlp", feature = "stdout")))]
            _ => Err(Error::Configuration(format!(
                "unsupported metrics exporter: {exporter:?}",
            )))?,
        }
    }

    Ok(builder.build())
}

#[macro_export]
macro_rules! counter {
    ($name:expr, $lvl:expr, $value:expr; $($fields:tt)*) => {
        __internal_paste! {
            tracing::event!(
                target: module_path!(),
                $lvl,
                counter.$name = $value,
                $($fields)*,
            )
        }
    };

    ($name:expr, $lvl:expr, $value:expr) => {
        $crate::counter!($name, $lvl, $value;)
    };

    ($name:expr, $lvl:expr; $($fields:tt)+) => {
        $crate::counter!($name, $lvl, 1; $($fields)+)
    };

    ($name:expr, $lvl:expr) => {
        $crate::counter!($name, $lvl, 1;)
    };
}

#[macro_export]
macro_rules! gauge {
    ($name:expr, $lvl:expr, $value:expr; $($fields:tt)*) => {
        __internal_paste! {
            tracing::event!(
                target: module_path!(),
                $lvl,
                gauge.$name = $value as f64,
                $($fields)*,
            )
        }
    };

    ($name:expr, $lvl:expr, $value:expr) => {
        $crate::gauge!($name, $lvl, $value;)
    };
}

#[macro_export]
macro_rules! histogram {
    ($name:expr, $lvl:expr, $value:expr; $($fields:tt)*) => {
        __internal_paste! {
            tracing::event!(
                target: module_path!(),
                $lvl,
                histogram.$name = $value as u64,
                $($fields)*,
            )
        }
    };

    ($name:expr, $lvl:expr, $value:expr) => {
        $crate::histogram!($name, $lvl;)
    };
}

#[macro_export]
macro_rules! monotonic_counter {
    ($name:expr, $lvl:expr, $value:expr; $($fields:tt)*) => {
        __internal_paste! {
            tracing::event!(
                target: module_path!(),
                $lvl,
                monotonic_counter.$name = $value,
                $($fields)*,
            )
        }
    };

    ($name:expr, $lvl:expr, $value:expr) => {
        $crate::monotonic_counter!($name, $lvl, $value;)
    };

    ($name:expr, $lvl:expr; $($fields:tt)+) => {
        $crate::monotonic_counter!($name, $lvl, 1; $($fields)+)
    };

    ($name:expr, $lvl:expr) => {
        $crate::monotonic_counter!($name, $lvl, 1;)
    };
}
