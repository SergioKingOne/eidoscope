//! Error types for stream processing

use eidoscope_ingestor::IngestorError;
use thiserror::Error;

/// Result type alias for stream processing operations
pub type Result<T> = std::result::Result<T, StreamError>;

/// Errors that can occur during stream processing
#[derive(Error, Debug)]
pub enum StreamError {
    /// Error occurred while consuming from the stream
    #[error("Stream consumption error: {0}")]
    ConsumptionError(String),

    /// Error occurred while processing a message through the pipeline
    #[error("Pipeline processing error: {source}")]
    PipelineError {
        #[from]
        source: IngestorError,
    },

    /// Stream consumer has been closed or disconnected
    #[error("Stream closed: {reason}")]
    StreamClosed { reason: String },

    /// Failed to commit offset or checkpoint
    #[error("Offset commit failed: {details}")]
    OffsetCommitFailed { details: String },

    /// Stream source is temporarily unavailable
    #[error("Stream unavailable: {source_name}")]
    StreamUnavailable { source_name: String },

    /// Configuration error
    #[error("Invalid configuration: {details}")]
    InvalidConfiguration { details: String },

    /// Shutdown signal received
    #[error("Shutdown requested")]
    ShutdownRequested,
}

impl StreamError {
    /// Creates a new consumption error
    pub fn consumption<S: Into<String>>(message: S) -> Self {
        Self::ConsumptionError(message.into())
    }

    /// Creates a new stream closed error
    pub fn closed<S: Into<String>>(reason: S) -> Self {
        Self::StreamClosed {
            reason: reason.into(),
        }
    }

    /// Creates a new offset commit error
    pub fn offset_commit_failed<S: Into<String>>(details: S) -> Self {
        Self::OffsetCommitFailed {
            details: details.into(),
        }
    }

    /// Creates a new stream unavailable error
    pub fn unavailable<S: Into<String>>(source_name: S) -> Self {
        Self::StreamUnavailable {
            source_name: source_name.into(),
        }
    }

    /// Creates a new invalid configuration error
    pub fn invalid_config<S: Into<String>>(details: S) -> Self {
        Self::InvalidConfiguration {
            details: details.into(),
        }
    }
}
