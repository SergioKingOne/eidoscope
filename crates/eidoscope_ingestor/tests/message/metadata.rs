//! Tests for Timestamp, SourceName, and MessageMetadata.

use eidoscope_ingestor::message::{MessageMetadata, SourceName, Timestamp};
use std::time::{Duration, SystemTime};

// ===== Timestamp Tests =====

#[test]
fn timestamp_now_creates_current_time() {
    let before = SystemTime::now();
    let timestamp = Timestamp::now();
    let after = SystemTime::now();

    let system_time = timestamp.as_system_time();
    assert!(system_time >= before);
    assert!(system_time <= after);
}

#[test]
fn timestamp_from_past_time_succeeds() {
    let past = SystemTime::now() - Duration::from_secs(3600);
    let result = Timestamp::from_system_time(past);
    assert!(result.is_ok());
}

#[test]
fn timestamp_from_recent_past_succeeds() {
    let past = SystemTime::now() - Duration::from_millis(100);
    let result = Timestamp::from_system_time(past);
    assert!(result.is_ok());
}

#[test]
fn timestamp_from_future_within_grace_period_succeeds() {
    // Within 1 second grace period
    let future = SystemTime::now() + Duration::from_millis(500);
    let result = Timestamp::from_system_time(future);
    assert!(result.is_ok());
}

#[test]
fn timestamp_from_future_beyond_grace_period_fails() {
    let future = SystemTime::now() + Duration::from_secs(2);
    let result = Timestamp::from_system_time(future);
    assert!(result.is_err());
    let error_str = result.unwrap_err().to_string();
    assert!(error_str.contains("Invalid timestamp"));
}

#[test]
fn timestamp_round_trip_conversion() {
    let original = SystemTime::now();
    let timestamp = Timestamp::from_system_time(original).unwrap();
    let converted = timestamp.as_system_time();
    assert_eq!(original, converted);
}

#[test]
fn timestamp_into_system_time() {
    let original = SystemTime::now();
    let timestamp = Timestamp::from_system_time(original).unwrap();
    let converted = timestamp.into_system_time();
    assert_eq!(original, converted);
}

#[test]
fn timestamp_ordering() {
    let t1 = SystemTime::now();
    std::thread::sleep(Duration::from_millis(10));
    let t2 = SystemTime::now();

    let ts1 = Timestamp::from_system_time(t1).unwrap();
    let ts2 = Timestamp::from_system_time(t2).unwrap();

    assert!(ts1 < ts2);
    assert!(ts2 > ts1);
}


// ===== SourceName Tests =====

#[test]
fn source_name_valid_alphanumeric() {
    let result = SourceName::new("service123");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_str(), "service123");
}

#[test]
fn source_name_with_hyphens() {
    let result = SourceName::new("my-service");
    assert!(result.is_ok());
}

#[test]
fn source_name_with_underscores() {
    let result = SourceName::new("my_service");
    assert!(result.is_ok());
}

#[test]
fn source_name_with_dots() {
    let result = SourceName::new("my.service.name");
    assert!(result.is_ok());
}

#[test]
fn source_name_with_slashes() {
    let result = SourceName::new("namespace/service");
    assert!(result.is_ok());
}

#[test]
fn source_name_rejects_empty_string() {
    let result = SourceName::new("");
    assert!(result.is_err());
    let error_str = result.unwrap_err().to_string();
    assert!(error_str.contains("cannot be empty"));
}

#[test]
fn source_name_rejects_whitespace_only() {
    let result = SourceName::new("   ");
    assert!(result.is_err());
    let error_str = result.unwrap_err().to_string();
    assert!(error_str.contains("cannot be empty"));
}

#[test]
fn source_name_accepts_max_length() {
    let long_name = "a".repeat(128);
    let result = SourceName::new(long_name.clone());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_str(), long_name);
}

#[test]
fn source_name_rejects_exceeding_max_length() {
    let too_long = "a".repeat(129);
    let result = SourceName::new(too_long);
    assert!(result.is_err());
    let error_str = result.unwrap_err().to_string();
    assert!(error_str.contains("exceeds maximum length"));
    assert!(error_str.contains("128"));
}

#[test]
fn source_name_rejects_invalid_characters() {
    let invalid_chars = vec!["service!", "service@home", "service#1", "service$", "service%"];

    for name in invalid_chars {
        let result = SourceName::new(name);
        assert!(result.is_err(), "Should reject: {}", name);
        let error_str = result.unwrap_err().to_string();
        assert!(error_str.contains("invalid format"));
    }
}

#[test]
fn source_name_rejects_whitespace() {
    let result = SourceName::new("my service");
    assert!(result.is_err());
    let error_str = result.unwrap_err().to_string();
    assert!(error_str.contains("invalid format"));
}

#[test]
fn source_name_rejects_newline() {
    let result = SourceName::new("service\nname");
    assert!(result.is_err());
    let error_str = result.unwrap_err().to_string();
    assert!(error_str.contains("invalid format"));
}

#[test]
fn source_name_into_string() {
    let name = SourceName::new("test-service").unwrap();
    let string = name.into_string();
    assert_eq!(string, "test-service");
}

// ===== MessageMetadata Tests =====

#[test]
fn message_metadata_new_uses_current_time() {
    let source = SourceName::new("test").unwrap();
    let before = SystemTime::now();
    let metadata = MessageMetadata::new(source.clone());
    let after = SystemTime::now();

    let timestamp_time = metadata.timestamp.as_system_time();
    assert!(timestamp_time >= before);
    assert!(timestamp_time <= after);
    assert_eq!(metadata.source.as_str(), "test");
}

#[test]
fn message_metadata_with_timestamp_preserves_time() {
    let source = SourceName::new("test").unwrap();
    let specific_time = SystemTime::now() - Duration::from_secs(3600);
    let timestamp = Timestamp::from_system_time(specific_time).unwrap();

    let metadata = MessageMetadata::with_timestamp(timestamp.clone(), source.clone());

    assert_eq!(metadata.timestamp, timestamp);
    assert_eq!(metadata.source.as_str(), "test");
}

