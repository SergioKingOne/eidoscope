//! In-memory channel-based stream consumer
//!
//! This module provides a [`ChannelConsumer`] implementation that consumes
//! messages from an in-memory channel. This is primarily useful for:
//! - Testing stream processing logic
//! - Local development
//! - Examples and demonstrations

use eidoscope_ingestor::message::Message;
use tokio::sync::mpsc;

use super::{ConsumedMessage, StreamConsumer};
use crate::error::{Result, StreamError};

/// A stream consumer that reads from an in-memory channel
///
/// This implementation uses Tokio's MPSC channel to simulate a message stream.
/// Messages are consumed from the receiver, and offsets are simulated using
/// a monotonically increasing counter.
///
/// # Example
///
/// ```rust
/// use eidoscope_ingestor::message::{GenericMessage, MessageId, MessageMetadata, SourceName};
/// use eidoscope_stream_processor::consumer::{StreamConsumer, channel::ChannelConsumer};
/// use tokio::sync::mpsc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let (tx, rx) = mpsc::channel(100);
/// let mut consumer = ChannelConsumer::new(rx);
///
/// // Send a message
/// let msg = GenericMessage::new(
///     MessageId::new("msg-1")?,
///     MessageMetadata::new(SourceName::new("test")?),
///     b"Hello".to_vec(),
///     Some("text/plain"),
/// )?;
/// tx.send(Box::new(msg)).await?;
///
/// // Consume the message
/// if let Some(consumed) = consumer.consume().await? {
///     println!("Consumed: {:?}", consumed.message.id());
/// }
/// # Ok(())
/// # }
/// ```
pub struct ChannelConsumer {
    receiver: mpsc::Receiver<Box<dyn Message>>,
    offset_counter: u64,
    source_name: String,
}

impl ChannelConsumer {
    /// Creates a new channel consumer from a receiver
    ///
    /// # Arguments
    ///
    /// * `receiver` - The MPSC receiver to consume messages from
    pub fn new(receiver: mpsc::Receiver<Box<dyn Message>>) -> Self {
        Self::with_name(receiver, "channel")
    }

    /// Creates a new channel consumer with a custom source name
    ///
    /// # Arguments
    ///
    /// * `receiver` - The MPSC receiver to consume messages from
    /// * `source_name` - The name to use for this stream source
    pub fn with_name<S: Into<String>>(
        receiver: mpsc::Receiver<Box<dyn Message>>,
        source_name: S,
    ) -> Self {
        Self {
            receiver,
            offset_counter: 0,
            source_name: source_name.into(),
        }
    }
}

impl StreamConsumer for ChannelConsumer {
    async fn consume(&mut self) -> Result<Option<ConsumedMessage>> {
        match self.receiver.recv().await {
            Some(message) => {
                self.offset_counter += 1;
                let offset = self.offset_counter.to_string();
                Ok(Some(ConsumedMessage::with_offset(message, offset)))
            }
            None => Ok(None), // Channel closed
        }
    }

    async fn commit(&mut self, _offset: &str) -> Result<()> {
        // For channel-based consumer, commit is a no-op since messages
        // are automatically removed from the channel once consumed
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        self.receiver.close();
        Ok(())
    }

    fn source_name(&self) -> &str {
        &self.source_name
    }
}

/// Builder for creating a channel consumer with a paired sender
///
/// This is a convenience for creating both ends of the channel at once.
///
/// # Example
///
/// ```rust
/// use eidoscope_stream_processor::consumer::channel::ChannelConsumerBuilder;
///
/// let (consumer, sender) = ChannelConsumerBuilder::new()
///     .with_capacity(100)
///     .with_source_name("my-stream")
///     .build();
/// ```
pub struct ChannelConsumerBuilder {
    capacity: usize,
    source_name: String,
}

impl ChannelConsumerBuilder {
    /// Creates a new builder with default settings
    ///
    /// Defaults:
    /// - Capacity: 32
    /// - Source name: "channel"
    pub fn new() -> Self {
        Self {
            capacity: 32,
            source_name: "channel".to_string(),
        }
    }

    /// Sets the channel capacity
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    /// Sets the source name for the consumer
    pub fn with_source_name<S: Into<String>>(mut self, source_name: S) -> Self {
        self.source_name = source_name.into();
        self
    }

    /// Builds the consumer and returns both the consumer and sender
    pub fn build(self) -> (ChannelConsumer, mpsc::Sender<Box<dyn Message>>) {
        let (tx, rx) = mpsc::channel(self.capacity);
        let consumer = ChannelConsumer::with_name(rx, self.source_name);
        (consumer, tx)
    }
}

impl Default for ChannelConsumerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eidoscope_ingestor::message::{GenericMessage, MessageId, MessageMetadata, SourceName};

    #[tokio::test]
    async fn test_channel_consumer_basic() {
        let (tx, rx) = mpsc::channel(10);
        let mut consumer = ChannelConsumer::new(rx);

        // Send a message
        let msg = GenericMessage::new(
            MessageId::new("msg-1").unwrap(),
            MessageMetadata::new(SourceName::new("test").unwrap()),
            b"Hello".to_vec(),
            Some("text/plain"),
        )
        .unwrap();
        tx.send(Box::new(msg)).await.unwrap();

        // Consume it
        let consumed = consumer.consume().await.unwrap();
        assert!(consumed.is_some());

        let consumed = consumed.unwrap();
        assert_eq!(consumed.message.id().as_str(), "msg-1");
        assert_eq!(consumed.offset, Some("1".to_string()));
    }

    #[tokio::test]
    async fn test_channel_consumer_closed() {
        let (tx, rx) = mpsc::channel::<Box<dyn Message>>(10);
        let mut consumer = ChannelConsumer::new(rx);

        // Close the sender
        drop(tx);

        // Should return None
        let result = consumer.consume().await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_channel_consumer_builder() {
        let (mut consumer, tx) = ChannelConsumerBuilder::new()
            .with_capacity(100)
            .with_source_name("test-stream")
            .build();

        assert_eq!(consumer.source_name(), "test-stream");

        let msg = GenericMessage::new(
            MessageId::new("msg-1").unwrap(),
            MessageMetadata::new(SourceName::new("test").unwrap()),
            b"Hello".to_vec(),
            None,
        )
        .unwrap();
        tx.send(Box::new(msg)).await.unwrap();

        let consumed = consumer.consume().await.unwrap();
        assert!(consumed.is_some());
    }
}
