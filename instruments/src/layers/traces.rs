use opentelemetry::{global, trace::TracerProvider as _};
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    trace::{SdkTracerProvider, Tracer},
    Resource,
};
use tracing::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::registry::LookupSpan;

use errors::prelude::*;

use crate::{exporters_from_env, Error, Exporter};

pub fn new_layer<S>(
    service_name: &'static str,
    resource: Resource,
) -> Result<OpenTelemetryLayer<S, Tracer>, Error>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let tracer_provider = new_provider(resource)?;
    global::set_text_map_propagator(TraceContextPropagator::new());
    global::set_tracer_provider(tracer_provider.clone());
    Ok(OpenTelemetryLayer::new(tracer_provider.tracer(service_name)))
}

fn new_provider(resource: Resource) -> Result<SdkTracerProvider, Error> {
    let mut builder = SdkTracerProvider::builder().with_resource(resource);

    for exporter in exporters_from_env("OTEL_TRACES_EXPORTER")? {
        match exporter {
            #[cfg(feature = "stdout")]
            Exporter::Console => {
                let exporter = opentelemetry_stdout::SpanExporter::default();
                builder = builder.with_simple_exporter(exporter);
            }

            Exporter::None => {
                // No-op
            }

            #[cfg(feature = "otlp")]
            Exporter::Otlp => {
                let exporter = opentelemetry_otlp::SpanExporter::builder()
                    .with_tonic()
                    .build()
                    .map_err(|v| Error::Internal(v.to_string()))?;

                builder = builder.with_batch_exporter(exporter);
            }

            #[cfg(not(all(feature = "otlp", feature = "stdout")))]
            _ => Err(Error::Configuration(format!(
                "unsupported traces exporter: {exporter:?}",
            )))?,
        }
    }

    Ok(builder.build())
}
