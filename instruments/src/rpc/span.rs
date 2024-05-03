use std::ops::Deref;

use tracing::Span;
use tracing_subscriber::{registry::LookupSpan, Registry};

use crate::layers::rpc::{RPCMethod, RPCService};

pub fn method_from_span(span: &Span) -> Option<String> {
    value_from_span::<RPCMethod>(span)
}

pub fn service_from_span(span: &Span) -> Option<String> {
    value_from_span::<RPCService>(span)
}

fn value_from_span<T>(span: &Span) -> Option<String>
where
    T: Deref<Target = String> + 'static,
{
    span.with_subscriber(|(id, subscriber)| {
        subscriber
            .downcast_ref::<Registry>()
            .and_then(|registry| registry.span(id))
            .and_then(|span| {
                span.extensions()
                    .get::<Option<T>>()
                    .and_then(|method| method.as_ref())
                    .map(|v| v.deref().clone())
            })
    })
    .flatten()
}
