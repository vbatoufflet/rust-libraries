use opentelemetry::{global, trace::TracerProvider as _};
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    trace::{Sampler, SdkTracerProvider, Tracer},
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
    exporter: &Exporter,
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

fn new_provider(resource: Resource, sample: f64, exporter: &Exporter) -> Result<SdkTracerProvider, Error> {
    let sampler = Sampler::TraceIdRatioBased(sample);

    #[allow(clippy::match_wildcard_for_single_variants)]
    let provider = match exporter {
        Exporter::Noop => SdkTracerProvider::builder()
            .with_resource(resource)
            .with_sampler(sampler)
            .build(),

        Exporter::Otlp => {
            let exporter = opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .build()
                .map_err(|v| Error::Internal(v.to_string()))?;

            SdkTracerProvider::builder()
                .with_resource(resource)
                .with_sampler(sampler)
                .with_batch_exporter(exporter)
                .build()
        }

        Exporter::Stdout => {
            let exporter = opentelemetry_stdout::SpanExporter::default();

            SdkTracerProvider::builder()
                .with_resource(resource)
                .with_sampler(sampler)
                .with_simple_exporter(exporter)
                .build()
        }

        _ => Err(Error::Configuration(format!(
            "unsupported exporter: {exporter:?}",
        )))?,
    };

    Ok(provider)
}
