//! Tests for MessageId.

use eidoscope_ingestor::message::MessageId;

#[test]
fn message_id_new_with_valid_id() {
    let id = MessageId::new("msg-123").unwrap();
    assert_eq!(id.as_str(), "msg-123");
}

#[test]
fn message_id_rejects_empty_string() {
    let result = MessageId::new("");
    assert!(result.is_err());
    let error_str = result.unwrap_err().to_string();
    assert!(error_str.contains("cannot be empty"));
}

#[test]
fn message_id_rejects_whitespace_only() {
    let result = MessageId::new("   ");
    assert!(result.is_err());
    let error_str = result.unwrap_err().to_string();
    assert!(error_str.contains("cannot be empty"));
}

#[test]
fn message_id_accepts_max_length() {
    let long_id = "a".repeat(256);
    let result = MessageId::new(long_id.clone());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_str(), long_id);
}

#[test]
fn message_id_rejects_exceeding_max_length() {
    let too_long_id = "a".repeat(257);
    let result = MessageId::new(too_long_id);
    assert!(result.is_err());
    let error_str = result.unwrap_err().to_string();
    assert!(error_str.contains("exceeds maximum length"));
}

#[test]
fn message_id_accepts_alphanumeric_and_common_chars() {
    let ids = vec![
        "msg-123",
        "msg_456",
        "msg.789",
        "msg:abc",
        "MSG-ABC-123",
        "uuid-550e8400-e29b-41d4-a716-446655440000",
    ];

    for id_str in ids {
        let result = MessageId::new(id_str);
        assert!(result.is_ok(), "Failed for ID: {}", id_str);
    }
}

#[test]
fn message_id_rejects_non_printable_characters() {
    let result = MessageId::new("msg\x00123");
    assert!(result.is_err());
    let error_str = result.unwrap_err().to_string();
    assert!(error_str.contains("invalid format"));
}

#[test]
fn message_id_rejects_newline() {
    let result = MessageId::new("msg\n123");
    assert!(result.is_err());
    let error_str = result.unwrap_err().to_string();
    assert!(error_str.contains("invalid format"));
}

#[test]
fn message_id_rejects_tab() {
    let result = MessageId::new("msg\t123");
    assert!(result.is_err());
    let error_str = result.unwrap_err().to_string();
    assert!(error_str.contains("invalid format"));
}

#[test]
fn message_id_new_or_generate_with_valid_id() {
    let id = MessageId::new_or_generate("msg-123");
    assert_eq!(id.as_str(), "msg-123");
}

#[test]
fn message_id_new_or_generate_generates_for_empty() {
    let id = MessageId::new_or_generate("");
    assert!(!id.as_str().is_empty());
    assert!(id.as_str().starts_with("msg-"));
}

#[test]
fn message_id_new_or_generate_generates_for_invalid() {
    let id = MessageId::new_or_generate("invalid\nid");
    assert!(!id.as_str().is_empty());
    assert!(id.as_str().starts_with("msg-"));
}

#[test]
fn message_id_generated_ids_are_unique() {
    let id1 = MessageId::new_or_generate("");
    // Small delay to ensure different timestamp
    std::thread::sleep(std::time::Duration::from_nanos(100));
    let id2 = MessageId::new_or_generate("");
    assert_ne!(id1.as_str(), id2.as_str());
}

#[test]
fn message_id_into_string_consumes_and_returns() {
    let id = MessageId::new("msg-123").unwrap();
    let string = id.into_string();
    assert_eq!(string, "msg-123");
}

#[test]
fn message_id_hash_consistency() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let id1 = MessageId::new("msg-123").unwrap();
    let id2 = MessageId::new("msg-123").unwrap();

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();

    id1.hash(&mut hasher1);
    id2.hash(&mut hasher2);

    assert_eq!(hasher1.finish(), hasher2.finish());
}
