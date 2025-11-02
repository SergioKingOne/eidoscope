//! Stream consumer abstractions and implementations
//!
//! This module defines the [`StreamConsumer`] trait which abstracts over different
//! message streaming sources (e.g., Kafka, Redis Streams, channels).

pub mod channel;

use eidoscope_ingestor::message::Message;
use std::future::Future;

use crate::error::Result;

/// Represents a consumed message with optional offset information
#[derive(Debug)]
pub struct ConsumedMessage {
    /// The message payload
    pub message: Box<dyn Message>,

    /// Optional offset/position in the stream for checkpointing
    pub offset: Option<String>,
}

impl ConsumedMessage {
    /// Creates a new consumed message without offset information
    pub fn new(message: Box<dyn Message>) -> Self {
        Self {
            message,
            offset: None,
        }
    }

    /// Creates a new consumed message with offset information
    pub fn with_offset<S: Into<String>>(message: Box<dyn Message>, offset: S) -> Self {
        Self {
            message,
            offset: Some(offset.into()),
        }
    }
}

/// Trait for consuming messages from a stream source
///
/// This trait abstracts over different streaming sources, allowing the same
/// processing logic to work with Kafka, Redis Streams, channels, or any
/// other message source.
///
/// # Object Safety
///
/// This trait is object-safe, allowing for dynamic dispatch and heterogeneous
/// collections of consumers.
///
/// # Example
///
/// ```rust,no_run
/// use eidoscope_stream_processor::consumer::StreamConsumer;
///
/// async fn process_stream<C: StreamConsumer>(mut consumer: C) {
///     while let Ok(Some(msg)) = consumer.consume().await {
///         // Process the message
///         println!("Received message: {:?}", msg.message.id());
///
///         // Commit the offset if available
///         if let Some(offset) = msg.offset {
///             consumer.commit(&offset).await.ok();
///         }
///     }
/// }
/// ```
pub trait StreamConsumer: Send {
    /// Consumes the next message from the stream
    ///
    /// Returns:
    /// - `Ok(Some(message))` if a message was successfully consumed
    /// - `Ok(None)` if the stream is closed gracefully
    /// - `Err(...)` if an error occurred during consumption
    fn consume(&mut self) -> impl Future<Output = Result<Option<ConsumedMessage>>> + Send;

    /// Commits/acknowledges a message offset
    ///
    /// This is used for checkpoint management to ensure messages are not
    /// reprocessed after a restart.
    ///
    /// # Arguments
    ///
    /// * `offset` - The offset identifier to commit
    fn commit(&mut self, offset: &str) -> impl Future<Output = Result<()>> + Send;

    /// Closes the stream consumer gracefully
    ///
    /// This should clean up any resources and ensure all pending commits
    /// are flushed.
    fn close(&mut self) -> impl Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }

    /// Returns the name of the stream source for logging/metrics
    fn source_name(&self) -> &str {
        "unknown"
    }
}
