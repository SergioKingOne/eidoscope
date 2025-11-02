//! Tests for message types.

mod id;
mod metadata;

use crate::common::fixtures::{test_message, test_source};
use eidoscope_ingestor::message::{GenericMessage, Message, MessageId};

// ===== Message Trait and GenericMessage Tests =====

#[test]
fn generic_message_new_creates_message() {
    let source = test_source();
    let payload = b"test data".to_vec();
    let message = GenericMessage::new(payload.clone(), source.clone());

    assert_eq!(message.payload(), payload.as_slice());
    assert_eq!(message.metadata().source.as_str(), source.as_str());
    assert!(message.content_type().is_none());
}

#[test]
fn generic_message_new_generates_id() {
    let source = test_source();
    let message = GenericMessage::new(b"data".to_vec(), source);

    assert!(!message.id().as_str().is_empty());
}

#[test]
fn generic_message_new_different_ids_for_different_messages() {
    let source = test_source();
    let message1 = GenericMessage::new(b"data1".to_vec(), source.clone());
    // Small delay to ensure different timestamp
    std::thread::sleep(std::time::Duration::from_nanos(100));
    let message2 = GenericMessage::new(b"data2".to_vec(), source);

    assert_ne!(message1.id().as_str(), message2.id().as_str());
}

#[test]
fn generic_message_with_id_uses_provided_id() {
    let source = test_source();
    let id = MessageId::new("custom-id-123").unwrap();
    let payload = b"data".to_vec();

    let message = GenericMessage::with_id(id.clone(), payload.clone(), source);

    assert_eq!(message.id().as_str(), "custom-id-123");
    assert_eq!(message.payload(), payload.as_slice());
}

#[test]
fn generic_message_with_empty_payload() {
    let source = test_source();
    let message = GenericMessage::new(Vec::new(), source);

    assert_eq!(message.payload(), &[]);
    assert_eq!(message.size(), 0);
}

#[test]
fn generic_message_with_large_payload() {
    let source = test_source();
    let large_payload = vec![0u8; 1024 * 1024]; // 1MB
    let message = GenericMessage::new(large_payload.clone(), source);

    assert_eq!(message.payload().len(), 1024 * 1024);
    assert_eq!(message.size(), 1024 * 1024);
}

#[test]
fn generic_message_with_content_type_builder() {
    let source = test_source();
    let message = GenericMessage::new(b"data".to_vec(), source)
        .with_content_type("application/json".to_string());

    assert_eq!(message.content_type(), Some("application/json"));
}

#[test]
fn generic_message_content_type_default_is_none() {
    let message = test_message();
    assert!(message.content_type().is_none());
}

#[test]
fn generic_message_size_matches_payload_length() {
    let source = test_source();
    let payload = b"test payload with various characters: 123!@#".to_vec();
    let message = GenericMessage::new(payload.clone(), source);

    assert_eq!(message.size(), payload.len());
}


#[test]
fn message_collection_with_different_types() {
    let source = test_source();
    let msg1 = GenericMessage::new(b"data1".to_vec(), source.clone());
    let msg2 =
        GenericMessage::new(b"data2".to_vec(), source).with_content_type("text/plain".to_string());

    let messages: Vec<Box<dyn Message>> = vec![Box::new(msg1), Box::new(msg2)];

    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].payload(), b"data1");
    assert_eq!(messages[1].content_type(), Some("text/plain"));
}

