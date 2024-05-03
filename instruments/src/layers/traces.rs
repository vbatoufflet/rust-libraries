use opentelemetry::{global, trace::TracerProvider as _};
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    runtime,
    trace::{Config, Sampler, Tracer, TracerProvider},
    Resource,
};
use tracing::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::registry::LookupSpan;

use errors::prelude::*;

use crate::{Error, Exporter};

pub fn new_layer<S>(
    service_name: &'static str,
    resource: Resource,
    exporter: Exporter,
    ratio_sample: f64,
) -> Result<OpenTelemetryLayer<S, Tracer>, Error>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let tracer_provider = new_provider(resource, ratio_sample, exporter)?;

    global::set_text_map_propagator(TraceContextPropagator::new());
    global::set_tracer_provider(tracer_provider.clone());

    Ok(OpenTelemetryLayer::new(tracer_provider.tracer(service_name)))
}

fn new_provider(resource: Resource, sample: f64, exporter: Exporter) -> Result<TracerProvider, Error> {
    let trace_config = Config::default()
        .with_sampler(Sampler::TraceIdRatioBased(sample))
        .with_resource(resource);

    let provider = match exporter {
        Exporter::Noop => TracerProvider::builder().with_config(trace_config).build(),

        Exporter::Otlp => {
            let exporter = opentelemetry_otlp::new_exporter()
                .tonic()
                .build_span_exporter()
                .map_err(|v| Error::Internal(v.to_string()))?;

            TracerProvider::builder()
                .with_config(trace_config)
                .with_batch_exporter(exporter, runtime::Tokio)
                .build()
        }

        Exporter::Stdout => {
            let exporter = opentelemetry_stdout::SpanExporter::builder().build();

            TracerProvider::builder()
                .with_config(trace_config)
                .with_simple_exporter(exporter)
                .build()
        }

        _ => Err(Error::Configuration(format!(
            "unsupported exporter: {:?}",
            exporter
        )))?,
    };

    Ok(provider)
}
