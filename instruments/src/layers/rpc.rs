use std::ops::Deref;

use tracing::{span, Subscriber};
use tracing_subscriber::{field::Visit, registry::LookupSpan, Layer};

pub const RPC_METHOD: &str = "rpc.method";
pub const RPC_SERVICE: &str = "rpc.service";

#[derive(Debug)]
pub struct RPCMethod(pub String);

impl Deref for RPCMethod {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct RPCService(pub String);

impl Deref for RPCService {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default)]
pub struct RPCLayer;

impl<S> Layer<S> for RPCLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    #[cfg(feature = "rpc")]
    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = RPCVisitor::default();
        attrs.record(&mut visitor);

        if let Some(span) = ctx.span(id) {
            let RPCVisitor {
                rpc_method,
                rpc_service,
            } = visitor;

            let extensions_mut = &mut span.extensions_mut();
            extensions_mut.insert(rpc_method.map(RPCMethod));
            extensions_mut.insert(rpc_service.map(RPCService));
        }
    }

    #[cfg(not(feature = "rpc"))]
    fn on_new_span(
        &self,
        _: &span::Attributes<'_>,
        _: &span::Id,
        _: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // noop
    }
}

#[derive(Default)]
pub struct RPCVisitor {
    rpc_method: Option<String>,
    rpc_service: Option<String>,
}

impl Visit for RPCVisitor {
    fn record_debug(&mut self, _: &tracing::field::Field, _: &dyn std::fmt::Debug) {}

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == RPC_METHOD {
            self.rpc_method = Some(value.to_owned())
        } else if field.name() == RPC_SERVICE {
            self.rpc_service = Some(value.to_owned())
        }
    }
}
