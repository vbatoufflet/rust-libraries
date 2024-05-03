use opentelemetry::global;
use opentelemetry_sdk::{
    metrics::{
        reader::{DefaultAggregationSelector, DefaultTemporalitySelector},
        PeriodicReader, SdkMeterProvider,
    },
    runtime, Resource,
};
use tracing::Subscriber;
use tracing_opentelemetry::MetricsLayer;
use tracing_subscriber::registry::LookupSpan;

use errors::prelude::*;

use crate::{Error, Exporter};

pub(crate) fn new_layer<S>(resource: Resource, exporter: Exporter) -> Result<MetricsLayer<S>, Error>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let meter_provider = new_provider(resource, exporter)?;

    global::set_meter_provider(meter_provider.clone());

    Ok(MetricsLayer::new(meter_provider))
}

fn new_provider(resource: Resource, exporter: Exporter) -> Result<SdkMeterProvider, Error> {
    let provider = match exporter {
        Exporter::Noop => SdkMeterProvider::builder().with_resource(resource).build(),

        Exporter::Otlp => {
            let exporter = opentelemetry_otlp::new_exporter()
                .tonic()
                .build_metrics_exporter(
                    Box::new(DefaultAggregationSelector::new()),
                    Box::new(DefaultTemporalitySelector::new()),
                )
                .map_err(|v| Error::Internal(v.to_string()))?;

            SdkMeterProvider::builder()
                .with_resource(resource)
                .with_reader(PeriodicReader::builder(exporter, runtime::Tokio).build())
                .build()
        }

        Exporter::Stdout => {
            let exporter = opentelemetry_stdout::MetricsExporter::builder().build();

            SdkMeterProvider::builder()
                .with_resource(resource)
                .with_reader(PeriodicReader::builder(exporter, runtime::Tokio).build())
                .build()
        }

        _ => Err(Error::Configuration(format!(
            "unsupported exporter: {:?}",
            exporter
        )))?,
    };

    Ok(provider)
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
