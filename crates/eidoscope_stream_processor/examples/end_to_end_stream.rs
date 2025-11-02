//! End-to-end example of stream processing
//!
//! This example demonstrates:
//! 1. Creating a stream source (using channels)
//! 2. Defining custom message processing logic
//! 3. Implementing a custom sink
//! 4. Running the stream processor
//! 5. Collecting and displaying statistics

use eidoscope_ingestor::{
    message::{GenericMessage, Message, MessageId, MessageMetadata, SourceName},
    pipeline::PipelineBuilder,
    processor::{ProcessingDecision, Processor, SkipReason},
    sink::Sink,
    IngestorError,
};
use eidoscope_stream_processor::{
    consumer::channel::ChannelConsumerBuilder, StreamConfig, StreamProcessor,
};
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::time::sleep;
use tracing::{info, Level};

/// A custom processor that validates message size and content
struct MessageValidator {
    max_size: usize,
}

impl MessageValidator {
    fn new(max_size: usize) -> Self {
        Self { max_size }
    }
}

impl Processor for MessageValidator {
    async fn process(&self, message: &dyn Message) -> eidoscope_ingestor::Result<ProcessingDecision> {
        // Check size
        if message.size() > self.max_size {
            return Ok(ProcessingDecision::Skip(SkipReason::Filtered {
                reason: format!(
                    "Message too large: {} bytes (max: {})",
                    message.size(),
                    self.max_size
                ),
            }));
        }

        // Check if payload is valid UTF-8
        if std::str::from_utf8(message.payload()).is_err() {
            return Ok(ProcessingDecision::Skip(SkipReason::Invalid {
                reason: "Payload is not valid UTF-8".to_string(),
            }));
        }

        // Check for empty messages
        if message.payload().is_empty() {
            return Ok(ProcessingDecision::Skip(SkipReason::Invalid {
                reason: "Empty payload".to_string(),
            }));
        }

        Ok(ProcessingDecision::Process)
    }
}

/// A custom sink that simulates processing and tracks metrics
#[derive(Clone)]
struct MetricsSink {
    processed_count: Arc<AtomicU64>,
    processing_delay: Duration,
}

impl MetricsSink {
    fn new(processing_delay: Duration) -> Self {
        Self {
            processed_count: Arc::new(AtomicU64::new(0)),
            processing_delay,
        }
    }

    fn count(&self) -> u64 {
        self.processed_count.load(Ordering::Relaxed)
    }
}

impl Sink for MetricsSink {
    async fn send(&self, message: Box<dyn Message>) -> eidoscope_ingestor::Result<()> {
        // Simulate some processing time
        sleep(self.processing_delay).await;

        let count = self.processed_count.fetch_add(1, Ordering::Relaxed) + 1;

        info!(
            message_id = %message.id(),
            source = %message.metadata().source_name(),
            size = message.size(),
            count = count,
            "Message processed"
        );

        Ok(())
    }

    async fn flush(&self) -> eidoscope_ingestor::Result<()> {
        info!(
            total_processed = self.count(),
            "Sink flushed"
        );
        Ok(())
    }

    fn name(&self) -> &str {
        "metrics-sink"
    }
}

/// Simulates a message producer that sends messages to the stream
async fn produce_messages(
    sender: tokio::sync::mpsc::Sender<Box<dyn Message>>,
    count: usize,
    source: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(count, source, "Starting message producer");

    for i in 0..count {
        let msg_id = format!("msg-{:04}", i);
        let payload = if i % 10 == 0 {
            // Every 10th message is intentionally too large
            "x".repeat(2000)
        } else if i % 7 == 0 {
            // Every 7th message is empty (will be filtered)
            String::new()
        } else {
            format!("Message number {} from {}", i, source)
        };

        let message = GenericMessage::new(
            MessageId::new(&msg_id)?,
            MessageMetadata::new(SourceName::new(source)?),
            payload.into_bytes(),
            Some("text/plain"),
        )?;

        sender.send(Box::new(message)).await?;

        // Simulate variable message arrival rate
        if i % 5 == 0 {
            sleep(Duration::from_millis(10)).await;
        }
    }

    info!(count, "Producer finished");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("Starting end-to-end stream processing example");

    // Create the channel consumer
    let (consumer, sender) = ChannelConsumerBuilder::new()
        .with_capacity(100)
        .with_source_name("example-stream")
        .build();

    // Create the processor (validates size < 1KB)
    let processor = MessageValidator::new(1024);

    // Create the sink with simulated processing delay
    let sink = MetricsSink::new(Duration::from_millis(5));
    let sink_clone = sink.clone();

    // Build the pipeline
    let pipeline = PipelineBuilder::new()
        .with_processor(processor)
        .with_sink(sink)
        .build()?;

    // Configure stream processing
    let config = StreamConfig::new()
        .with_auto_commit(true)
        .with_max_consecutive_failures(Some(10));

    // Create the stream processor
    let mut stream_processor = StreamProcessor::with_config(consumer, pipeline, config);

    // Spawn the message producer
    let producer_handle = tokio::spawn(async move {
        if let Err(e) = produce_messages(sender, 100, "example-source").await {
            eprintln!("Producer error: {}", e);
        }
    });

    // Run the stream processor
    info!("Starting stream processor");
    let stats = stream_processor.run().await?;

    // Wait for producer to finish
    producer_handle.await?;

    // Display final statistics
    info!("=== Stream Processing Complete ===");
    info!("Messages consumed:    {}", stats.consumed);
    info!("Messages processed:   {}", stats.processed);
    info!("Messages skipped:     {}", stats.skipped);
    info!("Messages failed:      {}", stats.failed);
    info!("Offsets committed:    {}", stats.committed);
    info!("Commit failures:      {}", stats.commit_failures);
    info!("Sink processed count: {}", sink_clone.count());

    // Verify counts match
    assert_eq!(
        stats.processed, sink_clone.count(),
        "Processed count mismatch"
    );

    info!("Example completed successfully!");

    Ok(())
}
