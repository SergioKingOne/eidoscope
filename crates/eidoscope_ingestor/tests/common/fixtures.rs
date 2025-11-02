//! Test fixtures and utilities.

use eidoscope_ingestor::message::{GenericMessage, SourceName};

/// Creates a test source name.
pub fn test_source() -> SourceName {
    SourceName::new("test-source").unwrap()
}

/// Creates a test message with default payload.
pub fn test_message() -> GenericMessage {
    GenericMessage::new(b"test payload".to_vec(), test_source())
}
