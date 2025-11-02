//! Passthrough processor implementation.

use super::{ProcessingDecision, Processor};
use crate::{error::Result, message::Message};

/// A processor that always allows messages through unchanged.
///
/// This is the identity processor that always returns [`ProcessingDecision::Process`].
/// It's useful for pipelines that need a processor but don't require any filtering
/// or transformation logic.
///
/// # Examples
///
/// ```
/// use eidoscope_ingestor::processor::{Processor, PassthroughProcessor, ProcessingDecision};
/// use eidoscope_ingestor::message::{GenericMessage, SourceName};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let processor = PassthroughProcessor;
/// let source = SourceName::new("test")?;
/// let message = GenericMessage::new(b"data".to_vec(), source);
///
/// let decision = processor.process(&message).await?;
/// assert!(decision.should_process());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct PassthroughProcessor;

impl Processor for PassthroughProcessor {
    async fn process(&self, _message: &dyn Message) -> Result<ProcessingDecision> {
        Ok(ProcessingDecision::Process)
    }
}
