use tower_http::classify::SharedClassifier;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

pub fn trace_layer() -> TraceLayer<SharedClassifier<tower_http::classify::ServerErrorsAsFailures>>
{
    TraceLayer::new_for_http()
}
