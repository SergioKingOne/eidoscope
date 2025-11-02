//! Tests for error types.

use eidoscope_ingestor::error::{
    IngestorError, ProcessingError, SinkError, ValidationError,
};

#[test]
fn validation_error_empty_value_display() {
    let error = ValidationError::EmptyValue {
        field: "message_id".to_string(),
    };
    assert_eq!(error.to_string(), "Field 'message_id' cannot be empty");
}

#[test]
fn validation_error_too_long_display() {
    let error = ValidationError::TooLong {
        field: "name".to_string(),
        max_length: 100,
        actual_length: 150,
    };
    assert_eq!(
        error.to_string(),
        "Field 'name' exceeds maximum length of 100 (got 150)"
    );
}

#[test]
fn validation_error_invalid_format_display() {
    let error = ValidationError::InvalidFormat {
        field: "source".to_string(),
        reason: "contains invalid characters".to_string(),
    };
    assert_eq!(
        error.to_string(),
        "Field 'source' has invalid format: contains invalid characters"
    );
}

#[test]
fn validation_error_out_of_range_display() {
    let error = ValidationError::OutOfRange {
        field: "priority".to_string(),
        value: 150,
        min: 0,
        max: 100,
    };
    assert_eq!(
        error.to_string(),
        "Field 'priority' value 150 is out of range [0, 100]"
    );
}

#[test]
fn validation_error_invalid_timestamp_display() {
    let error = ValidationError::InvalidTimestamp {
        field: "created_at".to_string(),
        reason: "timestamp is in the future".to_string(),
    };
    assert_eq!(
        error.to_string(),
        "Invalid timestamp for 'created_at': timestamp is in the future"
    );
}

#[test]
fn processing_error_aborted_display() {
    let error = ProcessingError::Aborted {
        reason: "user requested shutdown".to_string(),
    };
    assert_eq!(error.to_string(), "Processing aborted: user requested shutdown");
}

#[test]
fn processing_error_failed_display() {
    let error = ProcessingError::Failed {
        reason: "database connection lost".to_string(),
    };
    assert_eq!(error.to_string(), "Processing failed: database connection lost");
}

#[test]
fn processing_error_unexpected_state_display() {
    let error = ProcessingError::UnexpectedState {
        message: "processor in invalid state".to_string(),
    };
    assert_eq!(error.to_string(), "Unexpected state: processor in invalid state");
}

#[test]
fn sink_error_unavailable_display() {
    let error = SinkError::Unavailable {
        sink_name: "kafka-sink".to_string(),
    };
    assert_eq!(error.to_string(), "Sink 'kafka-sink' is unavailable");
}

#[test]
fn sink_error_rejected_display() {
    let error = SinkError::Rejected {
        reason: "invalid message format".to_string(),
    };
    assert_eq!(error.to_string(), "Message rejected: invalid message format");
}

#[test]
fn sink_error_internal_error_display() {
    let error = SinkError::InternalError {
        details: "buffer overflow".to_string(),
    };
    assert_eq!(error.to_string(), "Internal sink error: buffer overflow");
}

#[test]
fn sink_error_capacity_exceeded_display() {
    let error = SinkError::CapacityExceeded {
        current: 1500,
        maximum: 1000,
    };
    assert_eq!(error.to_string(), "Sink capacity exceeded: 1500/1000");
}

#[test]
fn ingestor_error_validation_variant_display() {
    let validation_error = ValidationError::EmptyValue {
        field: "test".to_string(),
    };
    let error = IngestorError::Validation(validation_error);
    assert_eq!(error.to_string(), "Validation error: Field 'test' cannot be empty");
}

#[test]
fn ingestor_error_processing_variant_display() {
    let processing_error = ProcessingError::Failed {
        reason: "test failure".to_string(),
    };
    let error = IngestorError::Processing(processing_error);
    assert_eq!(error.to_string(), "Processing error: Processing failed: test failure");
}

#[test]
fn ingestor_error_sink_variant_display() {
    let sink_error = SinkError::Unavailable {
        sink_name: "test-sink".to_string(),
    };
    let error = IngestorError::Sink(sink_error);
    assert_eq!(error.to_string(), "Sink error: Sink 'test-sink' is unavailable");
}

#[test]
fn ingestor_error_pipeline_construction_display() {
    let error = IngestorError::PipelineConstruction("missing processor".to_string());
    assert_eq!(error.to_string(), "Pipeline construction error: missing processor");
}

