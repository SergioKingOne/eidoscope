//! Message processing logic.
//!
//! This module provides traits and implementations for processing messages
//! in the ingestion pipeline. Processors decide whether messages should
//! continue to the sink or be skipped.
//!
//! # Async Design
//!
//! The `Processor` trait uses async methods. Implementers can choose to be
//! truly asynchronous or return ready futures for synchronous operations.

mod decision;
mod passthrough;

pub use decision::{ProcessingDecision, SkipReason};
pub use passthrough::PassthroughProcessor;

use crate::{error::Result, message::Message};
use std::future::Future;

/// Trait for message processors.
///
/// Processors examine messages and decide whether they should continue through
/// the pipeline to the sink or be skipped. The decision is encoded in the
/// [`ProcessingDecision`] type.
///
/// # Implementation Notes
///
/// The trait methods return futures, allowing both async and sync implementations.
/// For synchronous processors, you can return `std::future::ready()` or similar.
///
/// # Examples
///
/// Async implementation:
/// ```
/// use eidoscope_ingestor::{
///     processor::{Processor, ProcessingDecision, SkipReason},
///     message::Message,
/// };
///
/// struct AsyncSizeFilter {
///     max_size: usize,
/// }
///
/// impl Processor for AsyncSizeFilter {
///     async fn process(&self, message: &dyn Message) -> eidoscope_ingestor::error::Result<ProcessingDecision> {
///         // Could do async work here
///         if message.size() > self.max_size {
///             Ok(ProcessingDecision::Skip(SkipReason::filtered(
///                 format!("Message too large: {} bytes", message.size())
///             )))
///         } else {
///             Ok(ProcessingDecision::Process)
///         }
///     }
/// }
/// ```
///
/// Sync implementation using ready future:
/// ```
/// use eidoscope_ingestor::{
///     processor::{Processor, ProcessingDecision},
///     message::Message,
/// };
/// use std::future::ready;
///
/// struct SyncProcessor;
///
/// impl Processor for SyncProcessor {
///     async fn process(&self, _message: &dyn Message) -> eidoscope_ingestor::error::Result<ProcessingDecision> {
///         // Synchronous logic wrapped in async
///         Ok(ProcessingDecision::Process)
///     }
/// }
/// ```
pub trait Processor: Send + Sync {
    /// Processes a message and returns a decision.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to process
    ///
    /// # Returns
    ///
    /// A future resolving to a [`ProcessingDecision`] indicating whether
    /// to process or skip the message.
    ///
    /// # Errors
    ///
    /// Returns an error if processing fails in an unrecoverable way.
    /// For expected skip conditions, return `Ok(ProcessingDecision::Skip(_))`.
    fn process(
        &self,
        message: &dyn Message,
    ) -> impl Future<Output = Result<ProcessingDecision>> + Send;
}

/// A type-erased, boxed processor.
///
/// This allows for dynamic dispatch and heterogeneous collections of processors.
///
/// # Examples
///
/// ```
/// use eidoscope_ingestor::processor::{BoxedProcessor, PassthroughProcessor};
///
/// let processor: BoxedProcessor = Box::new(PassthroughProcessor);
/// ```
pub type BoxedProcessor = Box<dyn Processor>;
