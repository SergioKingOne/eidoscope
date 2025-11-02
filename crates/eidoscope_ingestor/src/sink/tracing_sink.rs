//! Tracing-based sink implementation.

use super::Sink;
use crate::{error::Result, message::Message};
use tracing::{info, Level};

/// A sink that logs messages using the `tracing` crate.
///
/// This sink is useful for debugging, testing, and observability. It outputs
/// structured log events for each message sent to it.
///
/// # Examples
///
/// ```
/// use eidoscope_ingestor::sink::{Sink, TracingSink};
/// use eidoscope_ingestor::message::{GenericMessage, SourceName};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let sink = TracingSink::new();
///
/// let source = SourceName::new("test")?;
/// let message = GenericMessage::new(b"data".to_vec(), source);
/// sink.send(Box::new(message)).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct TracingSink {
    level: Level,
}

impl Default for TracingSink {
    fn default() -> Self {
        Self::new()
    }
}

impl TracingSink {
    /// Creates a new `TracingSink` with INFO level logging.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::sink::TracingSink;
    ///
    /// let sink = TracingSink::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self { level: Level::INFO }
    }

    /// Creates a new `TracingSink` with a specific log level.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::sink::TracingSink;
    /// use tracing::Level;
    ///
    /// let sink = TracingSink::with_level(Level::DEBUG);
    /// ```
    #[must_use]
    pub const fn with_level(level: Level) -> Self {
        Self { level }
    }
}

impl Sink for TracingSink {
    async fn send(&self, message: Box<dyn Message>) -> Result<()> {
        let message_id = message.id().as_str();
        let source = message.metadata().source.as_str();
        let size = message.size();
        let content_type = message.content_type().unwrap_or("unknown");

        match self.level {
            Level::TRACE => tracing::trace!(
                message_id,
                source,
                size,
                content_type,
                "Message sent to sink"
            ),
            Level::DEBUG => tracing::debug!(
                message_id,
                source,
                size,
                content_type,
                "Message sent to sink"
            ),
            Level::INFO => info!(
                message_id,
                source,
                size,
                content_type,
                "Message sent to sink"
            ),
            Level::WARN => tracing::warn!(
                message_id,
                source,
                size,
                content_type,
                "Message sent to sink"
            ),
            Level::ERROR => tracing::error!(
                message_id,
                source,
                size,
                content_type,
                "Message sent to sink"
            ),
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "TracingSink"
    }
}
