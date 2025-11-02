//! Message sinking destinations.
//!
//! This module provides traits and implementations for sending processed messages
//! to their final destinations (streams, databases, files, etc.).
//!
//! # Async Design
//!
//! The `Sink` trait uses async methods. Implementers can choose to be
//! truly asynchronous or return ready futures for synchronous operations.

mod tracing_sink;

pub use tracing_sink::TracingSink;

use crate::{error::Result, message::Message};
use std::future::Future;

/// Trait for message sinks.
///
/// Sinks are the final destination for messages that pass through the processing
/// pipeline. They handle sending messages to external systems like message queues,
/// databases, files, or network endpoints.
///
/// # Implementation Notes
///
/// The trait methods return futures, allowing both async and sync implementations.
/// For synchronous sinks, you can simply use `async fn` that doesn't await anything.
///
/// # Examples
///
/// Async implementation:
/// ```
/// use eidoscope_ingestor::{
///     sink::Sink,
///     message::Message,
/// };
///
/// struct AsyncKafkaSink {
///     // ... fields
/// }
///
/// impl Sink for AsyncKafkaSink {
///     async fn send(&self, message: Box<dyn Message>) -> eidoscope_ingestor::error::Result<()> {
///         // Async send logic
///         Ok(())
///     }
/// }
/// ```
///
/// Sync implementation:
/// ```
/// use eidoscope_ingestor::{
///     sink::Sink,
///     message::{Message, GenericMessage, SourceName},
/// };
/// use std::sync::{Arc, Mutex};
///
/// struct MemorySink {
///     messages: Arc<Mutex<Vec<Vec<u8>>>>,
/// }
///
/// impl Sink for MemorySink {
///     async fn send(&self, message: Box<dyn Message>) -> eidoscope_ingestor::error::Result<()> {
///         let payload = message.payload().to_vec();
///         self.messages.lock().unwrap().push(payload);
///         Ok(())
///     }
/// }
/// ```
pub trait Sink: Send + Sync {
    /// Sends a message to the sink.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send (ownership is transferred)
    ///
    /// # Returns
    ///
    /// A future resolving to `Ok(())` if the message was successfully sent.
    ///
    /// # Errors
    ///
    /// Returns a [`SinkError`](crate::error::SinkError) if the message cannot be sent.
    fn send(&self, message: Box<dyn Message>) -> impl Future<Output = Result<()>> + Send;

    /// Optional flush operation for batching sinks.
    ///
    /// Sinks that buffer messages should implement this to force sending
    /// all buffered messages immediately.
    ///
    /// The default implementation does nothing (no-op).
    fn flush(&self) -> impl Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }

    /// Returns the name of this sink for logging and debugging.
    ///
    /// The default implementation returns the type name.
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

/// A type-erased, boxed sink.
///
/// Note: With RPITIT (Return Position Impl Trait In Traits), the `Sink` trait
/// is not dyn-compatible, so this type cannot be directly instantiated. Use concrete
/// sink types or generics instead.
///
/// # Examples
///
/// ```compile_fail
/// use eidoscope_ingestor::sink::{BoxedSink, TracingSink};
///
/// let sink: BoxedSink = Box::new(TracingSink::new());
/// ```
pub type BoxedSink = Box<dyn Sink>;
