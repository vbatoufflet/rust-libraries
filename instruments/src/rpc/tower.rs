use http::{header::HeaderMap, request::Request, response::Response};
use opentelemetry::trace::SpanKind;
use percent_encoding::percent_decode;
use tower_http::{
    classify::{GrpcErrorsAsFailures, GrpcFailureClass, SharedClassifier},
    trace::{DefaultOnBodyChunk, MakeSpan, OnEos, OnFailure, OnRequest, OnResponse, TraceLayer},
};
use tracing::{Level, Span};

use crate::{
    counter, histogram,
    prelude::__internal_paste,
    rpc::{method_from_span, service_from_span},
};

pub type GRPCTraceLayer = TraceLayer<
    SharedClassifier<GrpcErrorsAsFailures>,
    InstrumentsMakeSpan,
    InstrumentsOnRequest,
    InstrumentsOnResponse,
    DefaultOnBodyChunk,
    InstrumentsOnEos,
    InstrumentsOnFailure,
>;

pub fn trace_layer() -> GRPCTraceLayer {
    TraceLayer::new_for_grpc()
        .make_span_with(InstrumentsMakeSpan {})
        .on_request(InstrumentsOnRequest {})
        .on_response(InstrumentsOnResponse {})
        .on_eos(InstrumentsOnEos {})
        .on_failure(InstrumentsOnFailure {})
}

#[derive(Clone, Debug)]
pub struct InstrumentsMakeSpan;

impl<B> MakeSpan<B> for InstrumentsMakeSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let span_name = &request.uri().path()[1..];
        let parts = span_name.split_once("/");

        tracing::span!(
            Level::DEBUG,
            "request",
            otel.kind = format!("{:?}", SpanKind::Server),
            otel.name = span_name,
            rpc.method = parts.map(|(_, method)| method),
            rpc.service = parts.map(|(service, _)| service),
            rpc.system = "grpc",
        )
    }
}

#[derive(Clone, Debug)]
pub struct InstrumentsOnRequest;

impl<B> OnRequest<B> for InstrumentsOnRequest {
    fn on_request(&mut self, _request: &Request<B>, span: &Span) {
        tracing::debug!(
            rpc.method = method_from_span(span),
            rpc.service = service_from_span(span),
            "request processing started"
        );
    }
}

#[derive(Clone, Debug)]
pub struct InstrumentsOnResponse;

impl<B> OnResponse<B> for InstrumentsOnResponse {
    fn on_response(self, response: &Response<B>, latency: std::time::Duration, span: &Span) {
        let rpc_method = method_from_span(span);
        let rpc_service = service_from_span(span);

        let (rpc_code, rpc_message) = classification_from_headers(response.headers());

        histogram!(rpc_response_latency, Level::INFO, latency.as_millis();
            rpc.code = rpc_code,
            rpc.method = rpc_method,
            rpc.service = rpc_service,
        );

        tracing::debug!(
            rpc.method = rpc_method,
            rpc.code = rpc_code,
            rpc.message = rpc_message,
            rpc.service = rpc_service,
            latency = format_args!("{}ms", latency.as_millis()),
            "request processing finished"
        );
    }
}

#[derive(Clone, Debug)]
pub struct InstrumentsOnEos;

impl OnEos for InstrumentsOnEos {
    fn on_eos(self, _trailers: Option<&HeaderMap>, stream_duration: std::time::Duration, span: &Span) {
        let rpc_method = method_from_span(span);
        let rpc_service = service_from_span(span);

        histogram!(rpc_stream_duration, Level::INFO, stream_duration.as_millis();
            rpc.method = rpc_method,
            rpc.service = rpc_service,
        );

        tracing::debug!(
            rpc.method = method_from_span(span),
            stream_duration = format_args!("{}ms", stream_duration.as_millis()),
            "stream ended"
        );
    }
}

#[derive(Clone, Debug)]
pub struct InstrumentsOnFailure;

impl OnFailure<GrpcFailureClass> for InstrumentsOnFailure {
    fn on_failure(
        &mut self,
        failure_classification: GrpcFailureClass,
        latency: std::time::Duration,
        span: &Span,
    ) {
        let rpc_method = method_from_span(span);
        let rpc_service = service_from_span(span);

        let failure_code = match failure_classification {
            GrpcFailureClass::Code(code) => code.get(),
            GrpcFailureClass::Error(_) => 2, // Unknown
        };

        let rpc_code = code_name(failure_code);

        counter!(rpc_failure, Level::INFO;
            rpc.code = rpc_code,
            rpc.method = rpc_method,
            rpc.service = rpc_service,
        );

        match failure_code {
            2 | 8 | 12 | 13 | 14 => {
                tracing::error!(
                    rpc.code = rpc_code,
                    rpc.method = rpc_method,
                    rpc.service = rpc_service,
                    latency = format_args!("{}ms", latency.as_millis()),
                    "request failed"
                );
            }

            _ => {
                tracing::debug!(
                    rpc.code = rpc_code,
                    rpc.method = rpc_method,
                    rpc.service = rpc_service,
                    latency = format_args!("{}ms", latency.as_millis()),
                    "request failed"
                );
            }
        }
    }
}

fn classification_from_headers(headers: &HeaderMap) -> (Option<&str>, Option<String>) {
    let code = headers
        .get("grpc-status")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(0);

    let message = headers
        .get("grpc-message")
        .and_then(|v| percent_decode(v.as_bytes()).decode_utf8().ok())
        .map(|v| v.to_string());

    (code_name(code), message)
}

fn code_name(code: i32) -> Option<&'static str> {
    match code {
        0 => Some("ok"),
        1 => Some("cancelled"),
        2 => Some("unknown"),
        3 => Some("invalid_argument"),
        4 => Some("deadline_exceeded"),
        5 => Some("not_found"),
        6 => Some("already_exists"),
        7 => Some("permission_denied"),
        8 => Some("resource_exhausted"),
        9 => Some("failed_precondition"),
        10 => Some("aborted"),
        11 => Some("out_of_range"),
        12 => Some("unimplemented"),
        13 => Some("internal"),
        14 => Some("unavailable"),
        15 => Some("data_loss"),
        16 => Some("unauthenticated"),
        _ => None,
    }
}
