//! Tests for TracingSink.

use crate::common::fixtures::{test_message, test_source};
use eidoscope_ingestor::sink::{Sink, TracingSink};
use tracing::Level;

#[tokio::test]
async fn tracing_sink_new_defaults_to_info_level() {
    let sink = TracingSink::new();
    let message = test_message();

    let result = sink.send(Box::new(message)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn tracing_sink_with_level_trace() {
    let sink = TracingSink::with_level(Level::TRACE);
    let message = test_message();

    let result = sink.send(Box::new(message)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn tracing_sink_with_level_debug() {
    let sink = TracingSink::with_level(Level::DEBUG);
    let message = test_message();

    let result = sink.send(Box::new(message)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn tracing_sink_with_level_info() {
    let sink = TracingSink::with_level(Level::INFO);
    let message = test_message();

    let result = sink.send(Box::new(message)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn tracing_sink_with_level_warn() {
    let sink = TracingSink::with_level(Level::WARN);
    let message = test_message();

    let result = sink.send(Box::new(message)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn tracing_sink_with_level_error() {
    let sink = TracingSink::with_level(Level::ERROR);
    let message = test_message();

    let result = sink.send(Box::new(message)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn tracing_sink_handles_empty_payload() {
    let sink = TracingSink::new();
    let source = test_source();
    let message = eidoscope_ingestor::message::GenericMessage::new(Vec::new(), source);

    let result = sink.send(Box::new(message)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn tracing_sink_handles_large_payload() {
    let sink = TracingSink::new();
    let source = test_source();
    let large_payload = vec![0u8; 1024 * 1024];
    let message = eidoscope_ingestor::message::GenericMessage::new(large_payload, source);

    let result = sink.send(Box::new(message)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn tracing_sink_handles_message_with_content_type() {
    let sink = TracingSink::new();
    let source = test_source();
    let message = eidoscope_ingestor::message::GenericMessage::new(b"data".to_vec(), source)
        .with_content_type("application/json".to_string());

    let result = sink.send(Box::new(message)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn tracing_sink_handles_message_without_content_type() {
    let sink = TracingSink::new();
    let message = test_message();

    let result = sink.send(Box::new(message)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn tracing_sink_name() {
    let sink = TracingSink::new();
    assert_eq!(sink.name(), "TracingSink");
}

#[tokio::test]
async fn tracing_sink_flush_succeeds() {
    let sink = TracingSink::new();
    let result = sink.flush().await;
    assert!(result.is_ok());
}
