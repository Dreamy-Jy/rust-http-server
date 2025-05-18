use std::sync::{LazyLock, OnceLock, RwLock};

use opentelemetry::global::BoxedTracer;
use opentelemetry_sdk::{
    Resource, logs::SdkLoggerProvider, metrics::SdkMeterProvider, trace::SdkTracerProvider,
};

pub static SHUTDOWN_SERVER: RwLock<bool> = RwLock::new(false);
pub static METER_PROVIDER: OnceLock<SdkMeterProvider> = OnceLock::new();
pub static TRACER_PROVIDER: OnceLock<SdkTracerProvider> = OnceLock::new();
pub static LOGGER_PROVIDER: OnceLock<SdkLoggerProvider> = OnceLock::new();
pub static TELEMETRY_CONFIG: LazyLock<Resource> =
    LazyLock::new(|| Resource::builder().with_service_name("http_server").build());

pub static TRACER: OnceLock<BoxedTracer> = OnceLock::new();
