//! Message types and traits.
//!
//! This module defines the core [`Message`] trait that all messages must implement,
//! along with supporting types for message identification and metadata.
//!
//! # Design Philosophy
//!
//! The `Message` trait is object-safe, allowing for runtime polymorphism while
//! maintaining type safety. It follows the Rust API guideline of making traits
//! object-safe when they're likely to be used as trait objects (C-OBJECT).

mod id;
mod metadata;

pub use id::MessageId;
pub use metadata::{MessageMetadata, SourceName, Timestamp};

use std::fmt::Debug;

/// Core trait for all messages in the ingestion pipeline.
///
/// This trait defines the minimal interface that all message types must implement.
/// It is object-safe, allowing different message types to be used polymorphically.
///
/// # Object Safety
///
/// This trait is intentionally object-safe to support dynamic dispatch and
/// heterogeneous collections of messages (via `Box<dyn Message>`).
///
/// # Examples
///
/// ```
/// use eidoscope_ingestor::message::{Message, MessageId, MessageMetadata, SourceName};
/// use std::fmt;
///
/// #[derive(Debug)]
/// struct MyMessage {
///     id: MessageId,
///     metadata: MessageMetadata,
///     data: Vec<u8>,
/// }
///
/// impl Message for MyMessage {
///     fn id(&self) -> &MessageId {
///         &self.id
///     }
///
///     fn metadata(&self) -> &MessageMetadata {
///         &self.metadata
///     }
///
///     fn payload(&self) -> &[u8] {
///         &self.data
///     }
/// }
/// ```
pub trait Message: Debug + Send + Sync {
    /// Returns the unique identifier for this message.
    ///
    /// Message IDs should be stable and unique within the context of
    /// the ingestion system.
    fn id(&self) -> &MessageId;

    /// Returns the metadata associated with this message.
    ///
    /// Metadata includes information like timestamps and source identifiers.
    fn metadata(&self) -> &MessageMetadata;

    /// Returns the raw payload data of the message.
    ///
    /// This is the actual content being ingested, as a byte slice.
    /// The interpretation of this data is left to processors and sinks.
    fn payload(&self) -> &[u8];

    /// Returns an optional content type hint for the payload.
    ///
    /// This can be used by processors to determine how to interpret the payload.
    /// Common values might be "application/json", "text/plain", etc.
    ///
    /// The default implementation returns `None`.
    fn content_type(&self) -> Option<&str> {
        None
    }

    /// Returns the size of the message payload in bytes.
    ///
    /// The default implementation computes this from the payload,
    /// but implementations may override this for efficiency.
    fn size(&self) -> usize {
        self.payload().len()
    }
}

/// A concrete, general-purpose message implementation.
///
/// This is a simple implementation of the `Message` trait that can be used
/// for basic ingestion scenarios where a custom message type isn't needed.
///
/// # Examples
///
/// ```
/// use eidoscope_ingestor::message::{GenericMessage, Message, MessageId, SourceName};
///
/// let message = GenericMessage::new(
///     b"Hello, world!".to_vec(),
///     SourceName::new("test-source")?,
/// );
///
/// assert_eq!(message.payload(), b"Hello, world!");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct GenericMessage {
    id: MessageId,
    metadata: MessageMetadata,
    payload: Vec<u8>,
    content_type: Option<String>,
}

impl GenericMessage {
    /// Creates a new generic message with the given payload and source.
    ///
    /// A message ID is automatically generated, and the timestamp is set to the current time.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::message::{GenericMessage, SourceName};
    ///
    /// let source = SourceName::new("my-source")?;
    /// let message = GenericMessage::new(b"data".to_vec(), source);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn new(payload: Vec<u8>, source: SourceName) -> Self {
        Self {
            id: MessageId::new_or_generate(""),
            metadata: MessageMetadata::new(source),
            payload,
            content_type: None,
        }
    }

    /// Creates a new message with an explicit ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::message::{GenericMessage, MessageId, SourceName};
    ///
    /// let id = MessageId::new("msg-001")?;
    /// let source = SourceName::new("my-source")?;
    /// let message = GenericMessage::with_id(id, b"data".to_vec(), source);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn with_id(id: MessageId, payload: Vec<u8>, source: SourceName) -> Self {
        Self {
            id,
            metadata: MessageMetadata::new(source),
            payload,
            content_type: None,
        }
    }

    /// Creates a new message with complete control over all fields.
    #[must_use]
    pub fn with_metadata(
        id: MessageId,
        metadata: MessageMetadata,
        payload: Vec<u8>,
        content_type: Option<String>,
    ) -> Self {
        Self {
            id,
            metadata,
            payload,
            content_type,
        }
    }

    /// Sets the content type for this message.
    #[must_use]
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }
}

impl Message for GenericMessage {
    fn id(&self) -> &MessageId {
        &self.id
    }

    fn metadata(&self) -> &MessageMetadata {
        &self.metadata
    }

    fn payload(&self) -> &[u8] {
        &self.payload
    }

    fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }
}

/// Type alias for a boxed, dynamically-dispatched message.
///
/// This is useful when working with heterogeneous collections of messages
/// or when the concrete message type isn't known at compile time.
///
/// # Examples
///
/// ```
/// use eidoscope_ingestor::message::{BoxedMessage, GenericMessage, SourceName};
///
/// let source = SourceName::new("test")?;
/// let message = GenericMessage::new(b"data".to_vec(), source);
/// let boxed: BoxedMessage = Box::new(message);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub type BoxedMessage = Box<dyn Message>;
