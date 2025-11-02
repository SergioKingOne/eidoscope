//! Tests for stream processor

use eidoscope_ingestor::{
    message::{GenericMessage, Message, MessageId, MessageMetadata, SourceName},
    pipeline::PipelineBuilder,
    processor::{PassthroughProcessor, ProcessingDecision, Processor, SkipReason},
    sink::Sink,
};
use eidoscope_stream_processor::{
    consumer::channel::ChannelConsumerBuilder, StreamConfig, StreamProcessor,
};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

// Test sink that collects messages
#[derive(Clone)]
struct CollectorSink {
    messages: Arc<Mutex<Vec<String>>>,
}

impl CollectorSink {
    fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn collected(&self) -> Vec<String> {
        self.messages.lock().unwrap().clone()
    }
}

impl Sink for CollectorSink {
    async fn send(&self, message: Box<dyn Message>) -> eidoscope_ingestor::Result<()> {
        self.messages
            .lock()
            .unwrap()
            .push(message.id().as_str().to_string());
        Ok(())
    }

    fn name(&self) -> &str {
        "collector"
    }
}

// Test processor that filters based on message ID
struct FilterProcessor {
    prefix: String,
}

impl FilterProcessor {
    fn new(prefix: String) -> Self {
        Self { prefix }
    }
}

impl Processor for FilterProcessor {
    async fn process(
        &self,
        message: &dyn Message,
    ) -> eidoscope_ingestor::Result<ProcessingDecision> {
        if message.id().as_str().starts_with(&self.prefix) {
            Ok(ProcessingDecision::Process)
        } else {
            Ok(ProcessingDecision::Skip(SkipReason::Filtered {
                reason: format!("Does not start with '{}'", self.prefix),
            }))
        }
    }
}

fn create_test_message(id: &str) -> Box<dyn Message> {
    Box::new(
        GenericMessage::new(
            MessageId::new(id).unwrap(),
            MessageMetadata::new(SourceName::new("test").unwrap()),
            format!("payload-{}", id).into_bytes(),
            Some("text/plain"),
        )
        .unwrap(),
    )
}

#[tokio::test]
async fn test_stream_processor_basic() {
    let (consumer, tx) = ChannelConsumerBuilder::new().with_capacity(100).build();

    let sink = CollectorSink::new();
    let pipeline = PipelineBuilder::new()
        .with_processor(PassthroughProcessor)
        .with_sink(sink.clone())
        .build()
        .unwrap();

    let mut processor = StreamProcessor::new(consumer, pipeline);

    // Send some messages
    tx.send(create_test_message("msg-1")).await.unwrap();
    tx.send(create_test_message("msg-2")).await.unwrap();
    tx.send(create_test_message("msg-3")).await.unwrap();

    // Close the channel
    drop(tx);

    // Run the processor
    let stats = processor.run().await.unwrap();

    // Verify stats
    assert_eq!(stats.consumed, 3);
    assert_eq!(stats.processed, 3);
    assert_eq!(stats.skipped, 0);
    assert_eq!(stats.failed, 0);

    // Verify messages were collected
    let collected = sink.collected();
    assert_eq!(collected.len(), 3);
    assert!(collected.contains(&"msg-1".to_string()));
    assert!(collected.contains(&"msg-2".to_string()));
    assert!(collected.contains(&"msg-3".to_string()));
}

#[tokio::test]
async fn test_stream_processor_with_filtering() {
    let (consumer, tx) = ChannelConsumerBuilder::new().build();

    let sink = CollectorSink::new();
    let pipeline = PipelineBuilder::new()
        .with_processor(FilterProcessor::new("allowed-".to_string()))
        .with_sink(sink.clone())
        .build()
        .unwrap();

    let mut processor = StreamProcessor::new(consumer, pipeline);

    // Send messages - some will be filtered
    tx.send(create_test_message("allowed-1")).await.unwrap();
    tx.send(create_test_message("blocked-1")).await.unwrap();
    tx.send(create_test_message("allowed-2")).await.unwrap();
    tx.send(create_test_message("blocked-2")).await.unwrap();
    tx.send(create_test_message("allowed-3")).await.unwrap();

    drop(tx);

    let stats = processor.run().await.unwrap();

    // Verify stats
    assert_eq!(stats.consumed, 5);
    assert_eq!(stats.processed, 3);
    assert_eq!(stats.skipped, 2);
    assert_eq!(stats.failed, 0);

    // Verify only allowed messages were collected
    let collected = sink.collected();
    assert_eq!(collected.len(), 3);
    assert!(collected.contains(&"allowed-1".to_string()));
    assert!(collected.contains(&"allowed-2".to_string()));
    assert!(collected.contains(&"allowed-3".to_string()));
}

#[tokio::test]
async fn test_stream_processor_process_one() {
    let (consumer, tx) = ChannelConsumerBuilder::new().build();

    let sink = CollectorSink::new();
    let pipeline = PipelineBuilder::new()
        .with_processor(PassthroughProcessor)
        .with_sink(sink.clone())
        .build()
        .unwrap();

    let mut processor = StreamProcessor::new(consumer, pipeline);

    // Send a message
    tx.send(create_test_message("msg-1")).await.unwrap();

    // Process one message
    let decision = processor.process_one().await.unwrap();
    assert!(matches!(decision, Some(ProcessingDecision::Process)));

    // Verify stats
    let stats = processor.stats();
    assert_eq!(stats.consumed, 1);
    assert_eq!(stats.processed, 1);

    // Verify message was collected
    assert_eq!(sink.collected(), vec!["msg-1"]);

    // Send another and process
    tx.send(create_test_message("msg-2")).await.unwrap();
    let decision = processor.process_one().await.unwrap();
    assert!(matches!(decision, Some(ProcessingDecision::Process)));

    assert_eq!(processor.stats().consumed, 2);
    assert_eq!(processor.stats().processed, 2);
}

#[tokio::test]
async fn test_stream_processor_empty_stream() {
    let (consumer, tx) = ChannelConsumerBuilder::new().build();

    let sink = CollectorSink::new();
    let pipeline = PipelineBuilder::new()
        .with_processor(PassthroughProcessor)
        .with_sink(sink.clone())
        .build()
        .unwrap();

    let mut processor = StreamProcessor::new(consumer, pipeline);

    // Close immediately
    drop(tx);

    let stats = processor.run().await.unwrap();

    // Should handle empty stream gracefully
    assert_eq!(stats.consumed, 0);
    assert_eq!(stats.processed, 0);
    assert_eq!(sink.collected().len(), 0);
}

#[tokio::test]
async fn test_stream_config() {
    let config = StreamConfig::new()
        .with_auto_commit(false)
        .with_max_consecutive_failures(Some(5));

    assert!(!config.auto_commit);
    assert_eq!(config.max_consecutive_failures, Some(5));

    let (consumer, tx) = ChannelConsumerBuilder::new().build();
    let sink = CollectorSink::new();
    let pipeline = PipelineBuilder::new()
        .with_processor(PassthroughProcessor)
        .with_sink(sink)
        .build()
        .unwrap();

    let mut processor = StreamProcessor::with_config(consumer, pipeline, config);

    tx.send(create_test_message("msg-1")).await.unwrap();
    drop(tx);

    let stats = processor.run().await.unwrap();

    // With auto_commit disabled, committed should be 0
    assert_eq!(stats.consumed, 1);
    assert_eq!(stats.committed, 0);
}

#[tokio::test]
async fn test_stream_stats_tracking() {
    use eidoscope_stream_processor::StreamStats;

    let mut stats = StreamStats::new();
    assert_eq!(stats.consumed, 0);

    stats.inc_consumed();
    stats.inc_processed();
    assert_eq!(stats.consumed, 1);
    assert_eq!(stats.processed, 1);

    stats.inc_skipped();
    stats.inc_failed();
    assert_eq!(stats.skipped, 1);
    assert_eq!(stats.failed, 1);

    let mut other = StreamStats::new();
    other.inc_consumed();
    other.inc_consumed();

    stats.merge(&other);
    assert_eq!(stats.consumed, 3);
}
