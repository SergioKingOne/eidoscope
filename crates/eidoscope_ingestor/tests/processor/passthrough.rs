//! Tests for PassthroughProcessor.

use crate::common::fixtures::{test_message, test_source};
use eidoscope_ingestor::processor::{PassthroughProcessor, ProcessingDecision, Processor};

#[tokio::test]
async fn passthrough_processor_always_processes() {
    let processor = PassthroughProcessor;
    let message = test_message();

    let decision = processor.process(&message).await.unwrap();

    assert!(matches!(decision, ProcessingDecision::Process));
    assert!(decision.should_process());
    assert!(!decision.should_skip());
}

#[tokio::test]
async fn passthrough_processor_processes_empty_payload() {
    let processor = PassthroughProcessor;
    let source = test_source();
    let message = eidoscope_ingestor::message::GenericMessage::new(Vec::new(), source);

    let decision = processor.process(&message).await.unwrap();

    assert!(matches!(decision, ProcessingDecision::Process));
}

#[tokio::test]
async fn passthrough_processor_processes_large_payload() {
    let processor = PassthroughProcessor;
    let source = test_source();
    let large_payload = vec![0u8; 1024 * 1024];
    let message = eidoscope_ingestor::message::GenericMessage::new(large_payload, source);

    let decision = processor.process(&message).await.unwrap();

    assert!(matches!(decision, ProcessingDecision::Process));
}

#[tokio::test]
async fn passthrough_processor_multiple_messages() {
    let processor = PassthroughProcessor;

    for i in 0..10 {
        let source = test_source();
        let payload = format!("message {}", i).into_bytes();
        let message = eidoscope_ingestor::message::GenericMessage::new(payload, source);

        let decision = processor.process(&message).await.unwrap();
        assert!(matches!(decision, ProcessingDecision::Process));
    }
}

