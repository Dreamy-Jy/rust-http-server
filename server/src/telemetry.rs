use log::{LevelFilter, error, warn};
use opentelemetry::global::{self, BoxedTracer};
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_sdk::{
    error::OTelSdkError::{self, AlreadyShutdown, InternalFailure, Timeout},
    logs::{BatchLogProcessor, SdkLoggerProvider},
    metrics::SdkMeterProvider,
    trace::SdkTracerProvider,
};
use opentelemetry_stdout::{LogExporter, MetricExporter, SpanExporter};

use crate::statics::{LOGGER_PROVIDER, METER_PROVIDER, TELEMETRY_CONFIG, TRACER, TRACER_PROVIDER};

pub fn get_tracer() -> &'static BoxedTracer {
    TRACER.get_or_init(|| global::tracer("http_server"))
}

// this is a global flush function for all contexts
pub fn force_export_telemetry(is_retry: bool) {
    let mut should_retry = false;
    match METER_PROVIDER.get() {
        Some(meter_provider) => match meter_provider.force_flush() {
            Ok(_) | Err(AlreadyShutdown) => (),
            Err(Timeout(d)) => {
                should_retry = true;
                warn!(timeout = d.as_millis() ;"Force metric export timed out")
            }
            Err(InternalFailure(e)) => {
                error!(error = format!("{}", e).as_str(); "Internal failure occured force exporting metrics")
            }
        },
        None => {
            panic!("Metrics provider was not initialized");
        }
    };

    match TRACER_PROVIDER.get() {
        Some(tracer_provider) => match tracer_provider.force_flush() {
            Ok(_) | Err(AlreadyShutdown) => (),
            Err(Timeout(d)) => {
                should_retry = true;
                warn!(timeout = d.as_millis() ;"Force trace export timed out")
            }
            Err(InternalFailure(e)) => {
                error!(error = format!("{}", e).as_str(); "Internal failure occured force exporting traces")
            }
        },
        None => {
            panic!("Tracer provider was not initialized");
        }
    };

    match LOGGER_PROVIDER.get() {
        Some(logger_provider) => match logger_provider.force_flush() {
            Ok(_) | Err(AlreadyShutdown) => (),
            Err(Timeout(d)) => {
                should_retry = true;
                warn!(timeout = d.as_millis() ;"Force log export timed out")
            }
            Err(InternalFailure(e)) => {
                error!(error = format!("{}", e).as_str(); "Internal failure occured force exporting logs")
            }
        },
        None => {
            panic!("Logger provider was not initialized");
        }
    };

    if should_retry && !is_retry {
        force_export_telemetry(true);
    }
}

// this is an end of program grace shutdown function
pub fn shutdown_telemetry(
    logger: SdkLoggerProvider,
    meter: SdkMeterProvider,
    tracer: SdkTracerProvider,
) {
    force_export_telemetry(false); // possibly redundant | shutdown may export telemetry

    match logger.shutdown() {
        Ok(_) | Err(OTelSdkError::AlreadyShutdown) => (),
        Err(OTelSdkError::Timeout(d)) => {
            eprintln!(
                "Warning > Logger shutdown timed out | Timeout Duration: {:?}",
                d
            );
        }
        Err(OTelSdkError::InternalFailure(e)) => {
            eprintln!("Error > Logger shutdown failed | {}", e);
        }
    };

    match meter.shutdown() {
        Ok(_) | Err(OTelSdkError::AlreadyShutdown) => (),
        Err(OTelSdkError::Timeout(d)) => {
            eprintln!(
                "Warning > Meter shutdown timed out | Timeout Duration: {:?}",
                d
            );
        }
        Err(OTelSdkError::InternalFailure(e)) => {
            eprintln!("Error > Meter shutdown failed | {}", e);
        }
    };

    match tracer.shutdown() {
        Ok(_) | Err(OTelSdkError::AlreadyShutdown) => (),
        Err(OTelSdkError::Timeout(d)) => {
            eprintln!(
                "Warning > Tracer shutdown timed out | Timeout Duration: {:?}",
                d
            );
        }
        Err(OTelSdkError::InternalFailure(e)) => {
            eprintln!("Error > Tracer shutdown failed | {}", e);
        }
    };
}

pub fn init_telemetry() -> (SdkLoggerProvider, SdkMeterProvider, SdkTracerProvider) {
    return (init_logger(), init_meter(), init_tracer());
}

fn init_logger() -> SdkLoggerProvider {
    let log_exporter = LogExporter::default();
    let log_processor = BatchLogProcessor::builder(log_exporter).build();
    let logger_provider = SdkLoggerProvider::builder()
        .with_log_processor(log_processor)
        .with_resource(TELEMETRY_CONFIG.clone())
        .build();
    let log_bridge = OpenTelemetryLogBridge::new(&logger_provider);

    if let Err(e) = log::set_boxed_logger(Box::new(log_bridge)) {
        panic!("Couldn't set up logger | {}", e)
    }
    log::set_max_level(LevelFilter::max());

    if let Err(_) = LOGGER_PROVIDER.set(logger_provider.clone()) {
        panic!("Logger provider was already set");
    }

    return logger_provider;
}

fn init_meter() -> SdkMeterProvider {
    let metric_exporter = MetricExporter::default();
    let meter_provider = SdkMeterProvider::builder()
        .with_periodic_exporter(metric_exporter)
        .with_resource(TELEMETRY_CONFIG.clone())
        .build();

    global::set_meter_provider(meter_provider.clone());

    if let Err(_) = METER_PROVIDER.set(meter_provider.clone()) {
        panic!("Metrics provider was already set");
    }

    return meter_provider;
}

fn init_tracer() -> SdkTracerProvider {
    let span_exporter = SpanExporter::default();
    let tracer_provider = SdkTracerProvider::builder()
        .with_simple_exporter(span_exporter)
        .with_resource(TELEMETRY_CONFIG.clone())
        .build();
    global::set_tracer_provider(tracer_provider.clone());

    if let Err(_) = TRACER_PROVIDER.set(tracer_provider.clone()) {
        panic!("Trace provider was already set");
    }

    return tracer_provider;
}
