//! Pipeline orchestration for message ingestion.
//!
//! This module provides the core [`Pipeline`] type that orchestrates
//! the flow of messages through processors to sinks.

mod builder;

pub use builder::PipelineBuilder;

use crate::{
    error::Result,
    message::Message,
    processor::{ProcessingDecision, Processor},
    sink::Sink,
};
use tracing::{debug, warn};

/// Statistics about pipeline execution.
///
/// This provides observability into how many messages were processed,
/// skipped, or failed during ingestion.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PipelineStats {
    /// Total number of messages that entered the pipeline.
    pub received: u64,
    /// Number of messages successfully sent to the sink.
    pub processed: u64,
    /// Number of messages skipped by the processor.
    pub skipped: u64,
    /// Number of messages that encountered errors.
    pub failed: u64,
}

impl PipelineStats {
    /// Creates new stats with all counters at zero.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            received: 0,
            processed: 0,
            skipped: 0,
            failed: 0,
        }
    }

    /// Increments the received counter.
    pub const fn increment_received(&mut self) {
        self.received += 1;
    }

    /// Increments the processed counter.
    pub const fn increment_processed(&mut self) {
        self.processed += 1;
    }

    /// Increments the skipped counter.
    pub const fn increment_skipped(&mut self) {
        self.skipped += 1;
    }

    /// Increments the failed counter.
    pub const fn increment_failed(&mut self) {
        self.failed += 1;
    }
}

/// A complete ingestion pipeline.
///
/// The pipeline coordinates the flow of messages through a processor to a sink.
/// It handles the logic of calling the processor, checking the decision, and
/// routing messages appropriately.
///
/// # Type Parameters
///
/// * `P` - The processor type
/// * `S` - The sink type
///
/// # Design
///
/// This type uses the builder pattern for construction (see [`PipelineBuilder`]).
/// Once constructed, a pipeline is immutable and can be safely shared across threads.
///
/// # Examples
///
/// ```
/// use eidoscope_ingestor::{
///     pipeline::PipelineBuilder,
///     processor::PassthroughProcessor,
///     sink::TracingSink,
///     message::{GenericMessage, SourceName},
/// };
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pipeline = PipelineBuilder::new()
///     .with_processor(PassthroughProcessor)
///     .with_sink(TracingSink::new())
///     .build();
///
/// let source = SourceName::new("test")?;
/// let message = GenericMessage::new(b"data".to_vec(), source);
/// pipeline.ingest(Box::new(message)).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Pipeline<P, S>
where
    P: Processor,
    S: Sink,
{
    processor: P,
    sink: S,
}

impl<P, S> Pipeline<P, S>
where
    P: Processor,
    S: Sink,
{
    /// Creates a new pipeline with the given processor and sink.
    ///
    /// Prefer using [`PipelineBuilder`] for construction.
    #[must_use]
    pub const fn new(processor: P, sink: S) -> Self {
        Self { processor, sink }
    }

    /// Ingests a single message through the pipeline.
    ///
    /// This method:
    /// 1. Passes the message to the processor
    /// 2. Checks the processing decision
    /// 3. If the decision is to process, sends to the sink
    /// 4. If the decision is to skip, logs and returns success
    ///
    /// # Arguments
    ///
    /// * `message` - The message to ingest (ownership transferred)
    ///
    /// # Returns
    ///
    /// * `Ok(ProcessingDecision)` - The decision made by the processor
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The processor fails
    /// - The sink fails (only if decision was to process)
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::{
    ///     pipeline::PipelineBuilder,
    ///     processor::PassthroughProcessor,
    ///     sink::TracingSink,
    ///     message::{GenericMessage, SourceName},
    /// };
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pipeline = PipelineBuilder::new()
    ///     .with_processor(PassthroughProcessor)
    ///     .with_sink(TracingSink::new())
    ///     .build();
    ///
    /// let source = SourceName::new("test")?;
    /// let message = GenericMessage::new(b"test".to_vec(), source);
    /// let decision = pipeline.ingest(Box::new(message)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn ingest(&self, message: Box<dyn Message>) -> Result<ProcessingDecision> {
        let message_id = message.id().as_str().to_string();

        // Process the message
        let decision = self.processor.process(message.as_ref()).await?;

        match decision {
            ProcessingDecision::Process => {
                debug!(
                    message_id = %message_id,
                    sink = %self.sink.name(),
                    "Sending message to sink"
                );
                self.sink.send(message).await?;
            }
            ProcessingDecision::Skip(ref reason) => {
                debug!(
                    message_id = %message_id,
                    reason = %reason,
                    "Skipping message"
                );
            }
        }

        Ok(decision)
    }

    /// Ingests multiple messages and returns statistics.
    ///
    /// This is a convenience method for batch processing. It continues
    /// processing even if individual messages fail, collecting statistics
    /// about the entire batch.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::{
    ///     pipeline::PipelineBuilder,
    ///     processor::PassthroughProcessor,
    ///     sink::TracingSink,
    ///     message::{GenericMessage, SourceName, BoxedMessage},
    /// };
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pipeline = PipelineBuilder::new()
    ///     .with_processor(PassthroughProcessor)
    ///     .with_sink(TracingSink::new())
    ///     .build();
    ///
    /// let source = SourceName::new("test")?;
    /// let messages: Vec<BoxedMessage> = vec![
    ///     Box::new(GenericMessage::new(b"msg1".to_vec(), source.clone())),
    ///     Box::new(GenericMessage::new(b"msg2".to_vec(), source)),
    /// ];
    ///
    /// let stats = pipeline.ingest_batch(messages).await;
    /// assert_eq!(stats.received, 2);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn ingest_batch<I>(&self, messages: I) -> PipelineStats
    where
        I: IntoIterator<Item = Box<dyn Message>>,
    {
        let mut stats = PipelineStats::new();

        for message in messages {
            stats.increment_received();

            match self.ingest(message).await {
                Ok(ProcessingDecision::Process) => stats.increment_processed(),
                Ok(ProcessingDecision::Skip(_)) => stats.increment_skipped(),
                Err(e) => {
                    stats.increment_failed();
                    warn!(error = %e, "Failed to ingest message");
                }
            }
        }

        stats
    }

    /// Returns a reference to the processor.
    #[must_use]
    pub const fn processor(&self) -> &P {
        &self.processor
    }

    /// Returns a reference to the sink.
    #[must_use]
    pub const fn sink(&self) -> &S {
        &self.sink
    }

    /// Flushes any buffered data in the sink.
    ///
    /// # Errors
    ///
    /// Returns an error if the sink flush operation fails.
    pub async fn flush(&self) -> Result<()> {
        self.sink.flush().await
    }
}
