//! Error types for the ingestion service.
//!
//! This module provides comprehensive error handling for the ingestor,
//! following the Rust API guidelines for well-behaved error types.

use std::fmt;

/// The main error type for the eidoscope ingestor.
///
/// This type encompasses all possible errors that can occur during
/// message ingestion, processing, and sinking operations.
///
/// # Examples
///
/// ```
/// use eidoscope_ingestor::error::{IngestorError, ValidationError};
///
/// let error = IngestorError::Validation(ValidationError::EmptyValue {
///     field: "message_id".to_string(),
/// });
///
/// assert!(matches!(error, IngestorError::Validation(_)));
/// ```
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum IngestorError {
    /// Error during message validation.
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    /// Error during message processing.
    #[error("Processing error: {0}")]
    Processing(ProcessingError),

    /// Error when sending message to sink.
    #[error("Sink error: {0}")]
    Sink(SinkError),

    /// Error during pipeline construction.
    #[error("Pipeline construction error: {0}")]
    PipelineConstruction(String),
}

/// Validation errors for message and type construction.
///
/// These errors occur when attempting to construct types with invalid values,
/// implementing the "validated construction" pattern from type-driven design.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// A required field was empty or contained only whitespace.
    #[error("Field '{field}' cannot be empty")]
    EmptyValue {
        /// The name of the field that was empty.
        field: String,
    },

    /// A string value exceeded its maximum allowed length.
    #[error("Field '{field}' exceeds maximum length of {max_length} (got {actual_length})")]
    TooLong {
        /// The name of the field that exceeded the maximum length.
        field: String,
        /// The maximum allowed length.
        max_length: usize,
        /// The actual length of the value.
        actual_length: usize,
    },

    /// A value failed format validation (e.g., invalid characters, pattern mismatch).
    #[error("Field '{field}' has invalid format: {reason}")]
    InvalidFormat {
        /// The name of the field with invalid format.
        field: String,
        /// Description of why the format is invalid.
        reason: String,
    },

    /// A numeric value was out of acceptable range.
    #[error("Field '{field}' value {value} is out of range [{min}, {max}]")]
    OutOfRange {
        /// The name of the field with out-of-range value.
        field: String,
        /// The actual value that was out of range.
        value: i64,
        /// The minimum acceptable value.
        min: i64,
        /// The maximum acceptable value.
        max: i64,
    },

    /// A timestamp value was invalid (e.g., in the future when it shouldn't be).
    #[error("Invalid timestamp for '{field}': {reason}")]
    InvalidTimestamp {
        /// The name of the timestamp field.
        field: String,
        /// Description of why the timestamp is invalid.
        reason: String,
    },
}

/// Errors that occur during message processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessingError {
    /// Processing was aborted by the processor.
    Aborted {
        /// The reason processing was aborted.
        reason: String,
    },

    /// Processing failed due to an internal error.
    Failed {
        /// The reason processing failed.
        reason: String,
    },

    /// The processor encountered an unexpected state.
    UnexpectedState {
        /// Description of the unexpected state.
        message: String,
    },
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Aborted { reason } => write!(f, "Processing aborted: {reason}"),
            Self::Failed { reason } => write!(f, "Processing failed: {reason}"),
            Self::UnexpectedState { message } => write!(f, "Unexpected state: {message}"),
        }
    }
}

impl std::error::Error for ProcessingError {}

/// Errors that occur when sending messages to sinks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SinkError {
    /// The sink is currently unavailable.
    Unavailable {
        /// The name of the unavailable sink.
        sink_name: String,
    },

    /// The sink rejected the message.
    Rejected {
        /// The reason the message was rejected.
        reason: String,
    },

    /// The sink encountered an internal error.
    InternalError {
        /// Details about the internal error.
        details: String,
    },

    /// The sink's capacity was exceeded.
    CapacityExceeded {
        /// The current number of items in the sink.
        current: usize,
        /// The maximum capacity of the sink.
        maximum: usize,
    },
}

impl fmt::Display for SinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable { sink_name } => write!(f, "Sink '{sink_name}' is unavailable"),
            Self::Rejected { reason } => write!(f, "Message rejected: {reason}"),
            Self::InternalError { details } => write!(f, "Internal sink error: {details}"),
            Self::CapacityExceeded { current, maximum } => {
                write!(f, "Sink capacity exceeded: {current}/{maximum}")
            }
        }
    }
}

impl std::error::Error for SinkError {}

/// A specialized `Result` type for ingestor operations.
///
/// This type alias is used throughout the crate for operations
/// that can fail with an [`IngestorError`].
pub type Result<T> = std::result::Result<T, IngestorError>;
