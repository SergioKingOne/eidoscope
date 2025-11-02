//! Stream processor orchestration
//!
//! This module provides the [`StreamProcessor`] which orchestrates consuming
//! messages from a stream and processing them through an ingestion pipeline.

use eidoscope_ingestor::{
    pipeline::Pipeline,
    processor::{ProcessingDecision, Processor},
    sink::Sink,
};
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::{
    consumer::StreamConsumer,
    error::{Result, StreamError},
};

/// Statistics collected during stream processing
#[derive(Debug, Default, Clone, Copy)]
pub struct StreamStats {
    /// Total messages consumed from the stream
    pub consumed: u64,

    /// Messages successfully processed through the pipeline
    pub processed: u64,

    /// Messages skipped by the processor
    pub skipped: u64,

    /// Messages that failed processing
    pub failed: u64,

    /// Offsets successfully committed
    pub committed: u64,

    /// Failed offset commits
    pub commit_failures: u64,
}

impl StreamStats {
    /// Creates a new stats instance with all counters at zero
    pub fn new() -> Self {
        Self::default()
    }

    /// Increments the consumed counter
    pub fn inc_consumed(&mut self) {
        self.consumed += 1;
    }

    /// Increments the processed counter
    pub fn inc_processed(&mut self) {
        self.processed += 1;
    }

    /// Increments the skipped counter
    pub fn inc_skipped(&mut self) {
        self.skipped += 1;
    }

    /// Increments the failed counter
    pub fn inc_failed(&mut self) {
        self.failed += 1;
    }

    /// Increments the committed counter
    pub fn inc_committed(&mut self) {
        self.committed += 1;
    }

    /// Increments the commit failures counter
    pub fn inc_commit_failure(&mut self) {
        self.commit_failures += 1;
    }

    /// Merges another stats instance into this one
    pub fn merge(&mut self, other: &StreamStats) {
        self.consumed += other.consumed;
        self.processed += other.processed;
        self.skipped += other.skipped;
        self.failed += other.failed;
        self.committed += other.committed;
        self.commit_failures += other.commit_failures;
    }
}

/// Configuration for stream processing behavior
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Whether to auto-commit offsets after successful processing
    pub auto_commit: bool,

    /// Interval for periodic stats logging (None disables)
    pub stats_interval: Option<Duration>,

    /// Maximum number of consecutive failures before stopping
    pub max_consecutive_failures: Option<usize>,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            auto_commit: true,
            stats_interval: Some(Duration::from_secs(60)),
            max_consecutive_failures: Some(10),
        }
    }
}

impl StreamConfig {
    /// Creates a new configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to auto-commit offsets
    pub fn with_auto_commit(mut self, auto_commit: bool) -> Self {
        self.auto_commit = auto_commit;
        self
    }

    /// Sets the stats logging interval
    pub fn with_stats_interval(mut self, interval: Option<Duration>) -> Self {
        self.stats_interval = interval;
        self
    }

    /// Sets the maximum consecutive failures threshold
    pub fn with_max_consecutive_failures(mut self, max: Option<usize>) -> Self {
        self.max_consecutive_failures = max;
        self
    }
}

/// Orchestrates stream consumption and message processing
///
/// The `StreamProcessor` combines a [`StreamConsumer`] with a [`Pipeline`]
/// to continuously consume messages from a stream and process them.
///
/// # Type Parameters
///
/// * `C` - The stream consumer type
/// * `P` - The processor type
/// * `S` - The sink type
///
/// # Example
///
/// ```rust,no_run
/// use eidoscope_ingestor::{
///     processor::PassthroughProcessor,
///     sink::TracingSink,
///     pipeline::PipelineBuilder,
/// };
/// use eidoscope_stream_processor::{
///     consumer::channel::ChannelConsumerBuilder,
///     StreamProcessor,
/// };
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let (consumer, _sender) = ChannelConsumerBuilder::new().build();
///
/// let pipeline = PipelineBuilder::new()
///     .with_processor(PassthroughProcessor)
///     .with_sink(TracingSink::new(tracing::Level::INFO))
///     .build()?;
///
/// let mut processor = StreamProcessor::new(consumer, pipeline);
/// processor.run().await?;
/// # Ok(())
/// # }
/// ```
pub struct StreamProcessor<C, P, S>
where
    C: StreamConsumer,
    P: Processor,
    S: Sink,
{
    consumer: C,
    pipeline: Pipeline<P, S>,
    config: StreamConfig,
    stats: StreamStats,
}

impl<C, P, S> StreamProcessor<C, P, S>
where
    C: StreamConsumer,
    P: Processor,
    S: Sink,
{
    /// Creates a new stream processor with default configuration
    ///
    /// # Arguments
    ///
    /// * `consumer` - The stream consumer to read messages from
    /// * `pipeline` - The ingestion pipeline to process messages through
    pub fn new(consumer: C, pipeline: Pipeline<P, S>) -> Self {
        Self::with_config(consumer, pipeline, StreamConfig::default())
    }

    /// Creates a new stream processor with custom configuration
    ///
    /// # Arguments
    ///
    /// * `consumer` - The stream consumer to read messages from
    /// * `pipeline` - The ingestion pipeline to process messages through
    /// * `config` - Configuration for stream processing behavior
    pub fn with_config(consumer: C, pipeline: Pipeline<P, S>, config: StreamConfig) -> Self {
        Self {
            consumer,
            pipeline,
            config,
            stats: StreamStats::new(),
        }
    }

    /// Returns the current statistics
    pub fn stats(&self) -> &StreamStats {
        &self.stats
    }

    /// Runs the stream processor until the stream is closed or an error occurs
    ///
    /// This method will continuously consume messages from the stream and
    /// process them through the pipeline until:
    /// - The stream is closed
    /// - An unrecoverable error occurs
    /// - The maximum consecutive failures threshold is reached
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Stream consumption fails
    /// - Maximum consecutive failures is exceeded
    pub async fn run(&mut self) -> Result<StreamStats> {
        info!(
            source = self.consumer.source_name(),
            "Starting stream processor"
        );

        let mut consecutive_failures = 0;

        loop {
            match self.process_next().await {
                Ok(should_continue) => {
                    consecutive_failures = 0;
                    if !should_continue {
                        info!("Stream closed, shutting down processor");
                        break;
                    }
                }
                Err(e) => {
                    consecutive_failures += 1;
                    error!(
                        error = %e,
                        consecutive_failures,
                        "Error processing message"
                    );

                    if let Some(max) = self.config.max_consecutive_failures {
                        if consecutive_failures >= max {
                            error!(
                                max_failures = max,
                                "Maximum consecutive failures reached, stopping processor"
                            );
                            return Err(StreamError::consumption(format!(
                                "Exceeded maximum consecutive failures: {}",
                                max
                            )));
                        }
                    }
                }
            }
        }

        // Flush the pipeline before shutdown
        info!("Flushing pipeline before shutdown");
        self.pipeline
            .flush()
            .await
            .map_err(|e| StreamError::consumption(format!("Failed to flush pipeline: {}", e)))?;

        // Close the consumer
        info!("Closing stream consumer");
        self.consumer.close().await?;

        info!(
            consumed = self.stats.consumed,
            processed = self.stats.processed,
            skipped = self.stats.skipped,
            failed = self.stats.failed,
            committed = self.stats.committed,
            "Stream processor shutdown complete"
        );

        Ok(self.stats)
    }

    /// Processes the next message from the stream
    ///
    /// Returns `Ok(true)` if processing should continue, `Ok(false)` if the
    /// stream is closed, or an error if processing failed.
    async fn process_next(&mut self) -> Result<bool> {
        // Consume the next message
        let consumed = match self.consumer.consume().await? {
            Some(msg) => msg,
            None => return Ok(false), // Stream closed
        };

        self.stats.inc_consumed();
        debug!(
            message_id = %consumed.message.id(),
            offset = ?consumed.offset,
            "Consumed message from stream"
        );

        // Process through the pipeline
        match self.pipeline.ingest(consumed.message).await {
            Ok(ProcessingDecision::Process) => {
                self.stats.inc_processed();
                debug!("Message processed successfully");

                // Auto-commit if enabled and offset is available
                if self.config.auto_commit {
                    if let Some(offset) = &consumed.offset {
                        match self.consumer.commit(offset).await {
                            Ok(()) => {
                                self.stats.inc_committed();
                                debug!(offset = %offset, "Offset committed");
                            }
                            Err(e) => {
                                self.stats.inc_commit_failure();
                                warn!(
                                    offset = %offset,
                                    error = %e,
                                    "Failed to commit offset"
                                );
                            }
                        }
                    }
                }
            }
            Ok(ProcessingDecision::Skip(reason)) => {
                self.stats.inc_skipped();
                debug!(reason = %reason, "Message skipped");

                // Still commit the offset for skipped messages to avoid reprocessing
                if self.config.auto_commit {
                    if let Some(offset) = &consumed.offset {
                        match self.consumer.commit(offset).await {
                            Ok(()) => self.stats.inc_committed(),
                            Err(e) => {
                                self.stats.inc_commit_failure();
                                warn!(error = %e, "Failed to commit offset for skipped message");
                            }
                        }
                    }
                }
            }
            Err(e) => {
                self.stats.inc_failed();
                error!(error = %e, "Pipeline processing failed");
                // Don't commit offset on failure to allow reprocessing
                return Err(e.into());
            }
        }

        Ok(true)
    }

    /// Processes a single message and returns the decision
    ///
    /// This is useful for manual control over the processing loop.
    pub async fn process_one(&mut self) -> Result<Option<ProcessingDecision>> {
        match self.consumer.consume().await? {
            Some(consumed) => {
                self.stats.inc_consumed();
                let decision = self.pipeline.ingest(consumed.message).await?;

                match decision {
                    ProcessingDecision::Process => self.stats.inc_processed(),
                    ProcessingDecision::Skip(_) => self.stats.inc_skipped(),
                }

                if self.config.auto_commit {
                    if let Some(offset) = &consumed.offset {
                        match self.consumer.commit(offset).await {
                            Ok(()) => self.stats.inc_committed(),
                            Err(_) => self.stats.inc_commit_failure(),
                        }
                    }
                }

                Ok(Some(decision))
            }
            None => Ok(None),
        }
    }
}
