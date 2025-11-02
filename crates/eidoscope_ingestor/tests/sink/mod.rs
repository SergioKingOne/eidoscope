//! Tests for sink types.

mod tracing_sink;

use crate::common::{
    fixtures::{test_message, test_source},
    sinks::{CollectingSink, CountingSink, FailingSink},
};
use eidoscope_ingestor::{
    error::{IngestorError, SinkError},
    sink::Sink,
};

// ===== Sink Trait Tests =====

#[tokio::test]
async fn sink_trait_counting_sink() {
    let sink = CountingSink::new();
    assert_eq!(sink.count(), 0);

    let message = test_message();
    sink.send(Box::new(message)).await.unwrap();

    assert_eq!(sink.count(), 1);
}

#[tokio::test]
async fn sink_trait_counting_sink_multiple_messages() {
    let sink = CountingSink::new();

    for i in 0..10 {
        let source = test_source();
        let payload = format!("message {}", i).into_bytes();
        let message = eidoscope_ingestor::message::GenericMessage::new(payload, source);
        sink.send(Box::new(message)).await.unwrap();
    }

    assert_eq!(sink.count(), 10);
}

#[tokio::test]
async fn sink_trait_collecting_sink() {
    let sink = CollectingSink::new();

    let source = test_source();
    let payload = b"test data".to_vec();
    let message = eidoscope_ingestor::message::GenericMessage::new(payload.clone(), source);

    sink.send(Box::new(message)).await.unwrap();

    let messages = sink.messages();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0], payload);
}

#[tokio::test]
async fn sink_trait_collecting_sink_preserves_order() {
    let sink = CollectingSink::new();

    for i in 0..5 {
        let source = test_source();
        let payload = format!("message {}", i).into_bytes();
        let message = eidoscope_ingestor::message::GenericMessage::new(payload, source);
        sink.send(Box::new(message)).await.unwrap();
    }

    let messages = sink.messages();
    assert_eq!(messages.len(), 5);
    assert_eq!(messages[0], b"message 0");
    assert_eq!(messages[4], b"message 4");
}

#[tokio::test]
async fn sink_trait_failing_sink() {
    let sink = FailingSink::new(SinkError::Unavailable {
        sink_name: "test-sink".to_string(),
    });

    let message = test_message();
    let result = sink.send(Box::new(message)).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), IngestorError::Sink(_)));
}

#[tokio::test]
async fn sink_trait_failing_sink_with_different_errors() {
    let errors = vec![
        SinkError::Unavailable {
            sink_name: "test".to_string(),
        },
        SinkError::Rejected {
            reason: "test".to_string(),
        },
        SinkError::InternalError {
            details: "test".to_string(),
        },
        SinkError::CapacityExceeded {
            current: 100,
            maximum: 50,
        },
    ];

    for error in errors {
        let sink = FailingSink::new(error);
        let message = test_message();
        let result = sink.send(Box::new(message)).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IngestorError::Sink(_)));
    }
}

#[tokio::test]
async fn sink_trait_flush_default_impl() {
    let sink = CountingSink::new();
    let result = sink.flush().await;
    assert!(result.is_ok());
}

