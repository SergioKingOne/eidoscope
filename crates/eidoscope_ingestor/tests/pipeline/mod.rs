//! Tests for pipeline types.

mod builder;

use crate::common::{
    fixtures::{test_message, test_source},
    processors::{AlwaysProcessProcessor, AlwaysSkipProcessor, FailingProcessor},
    sinks::{CollectingSink, CountingSink, FailingSink},
};
use eidoscope_ingestor::{
    error::{ProcessingError, SinkError},
    pipeline::{PipelineBuilder, PipelineStats},
    processor::{ProcessingDecision, SkipReason},
};

// ===== PipelineStats Tests =====

#[test]
fn pipeline_stats_new_zeros_all_counters() {
    let stats = PipelineStats::new();

    assert_eq!(stats.received, 0);
    assert_eq!(stats.processed, 0);
    assert_eq!(stats.skipped, 0);
    assert_eq!(stats.failed, 0);
}

#[test]
fn pipeline_stats_default_zeros_all_counters() {
    let stats = PipelineStats::default();

    assert_eq!(stats.received, 0);
    assert_eq!(stats.processed, 0);
    assert_eq!(stats.skipped, 0);
    assert_eq!(stats.failed, 0);
}

#[test]
fn pipeline_stats_increment_received() {
    let mut stats = PipelineStats::new();
    stats.increment_received();

    assert_eq!(stats.received, 1);
    assert_eq!(stats.processed, 0);
    assert_eq!(stats.skipped, 0);
    assert_eq!(stats.failed, 0);
}

#[test]
fn pipeline_stats_increment_processed() {
    let mut stats = PipelineStats::new();
    stats.increment_processed();

    assert_eq!(stats.received, 0);
    assert_eq!(stats.processed, 1);
    assert_eq!(stats.skipped, 0);
    assert_eq!(stats.failed, 0);
}

#[test]
fn pipeline_stats_increment_skipped() {
    let mut stats = PipelineStats::new();
    stats.increment_skipped();

    assert_eq!(stats.received, 0);
    assert_eq!(stats.processed, 0);
    assert_eq!(stats.skipped, 1);
    assert_eq!(stats.failed, 0);
}

#[test]
fn pipeline_stats_increment_failed() {
    let mut stats = PipelineStats::new();
    stats.increment_failed();

    assert_eq!(stats.received, 0);
    assert_eq!(stats.processed, 0);
    assert_eq!(stats.skipped, 0);
    assert_eq!(stats.failed, 1);
}

#[test]
fn pipeline_stats_multiple_increments() {
    let mut stats = PipelineStats::new();

    stats.increment_received();
    stats.increment_received();
    stats.increment_received();
    stats.increment_processed();
    stats.increment_skipped();
    stats.increment_failed();

    assert_eq!(stats.received, 3);
    assert_eq!(stats.processed, 1);
    assert_eq!(stats.skipped, 1);
    assert_eq!(stats.failed, 1);
}

// ===== Pipeline Core Tests =====

#[tokio::test]
async fn pipeline_ingest_process_decision_sends_to_sink() {
    let sink = CountingSink::new();
    let pipeline = PipelineBuilder::new()
        .with_processor(AlwaysProcessProcessor)
        .with_sink(sink.clone())
        .build();

    let message = test_message();
    let decision = pipeline.ingest(Box::new(message)).await.unwrap();

    assert!(matches!(decision, ProcessingDecision::Process));
    assert_eq!(sink.count(), 1);
}

#[tokio::test]
async fn pipeline_ingest_skip_decision_does_not_send_to_sink() {
    let sink = CountingSink::new();
    let processor = AlwaysSkipProcessor::new(SkipReason::Duplicate);
    let pipeline = PipelineBuilder::new()
        .with_processor(processor)
        .with_sink(sink.clone())
        .build();

    let message = test_message();
    let decision = pipeline.ingest(Box::new(message)).await.unwrap();

    assert!(matches!(decision, ProcessingDecision::Skip(_)));
    assert_eq!(sink.count(), 0);
}

#[tokio::test]
async fn pipeline_ingest_processor_error_propagates() {
    let sink = CountingSink::new();
    let processor = FailingProcessor::new(ProcessingError::Failed {
        reason: "test error".to_string(),
    });
    let pipeline = PipelineBuilder::new()
        .with_processor(processor)
        .with_sink(sink.clone())
        .build();

    let message = test_message();
    let result = pipeline.ingest(Box::new(message)).await;

    assert!(result.is_err());
    assert_eq!(sink.count(), 0);
}

#[tokio::test]
async fn pipeline_ingest_sink_error_propagates() {
    let sink = FailingSink::new(SinkError::Unavailable {
        sink_name: "test".to_string(),
    });
    let pipeline = PipelineBuilder::new()
        .with_processor(AlwaysProcessProcessor)
        .with_sink(sink)
        .build();

    let message = test_message();
    let result = pipeline.ingest(Box::new(message)).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn pipeline_ingest_collects_payloads() {
    let sink = CollectingSink::new();
    let pipeline = PipelineBuilder::new()
        .with_processor(AlwaysProcessProcessor)
        .with_sink(sink.clone())
        .build();

    let source = test_source();
    let payload1 = b"message 1".to_vec();
    let payload2 = b"message 2".to_vec();

    let message1 =
        eidoscope_ingestor::message::GenericMessage::new(payload1.clone(), source.clone());
    let message2 = eidoscope_ingestor::message::GenericMessage::new(payload2.clone(), source);

    pipeline.ingest(Box::new(message1)).await.unwrap();
    pipeline.ingest(Box::new(message2)).await.unwrap();

    let messages = sink.messages();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0], payload1);
    assert_eq!(messages[1], payload2);
}

#[tokio::test]
async fn pipeline_ingest_batch_empty() {
    let sink = CountingSink::new();
    let pipeline = PipelineBuilder::new()
        .with_processor(AlwaysProcessProcessor)
        .with_sink(sink.clone())
        .build();

    let messages: Vec<Box<dyn eidoscope_ingestor::message::Message>> = vec![];
    let stats = pipeline.ingest_batch(messages).await;

    assert_eq!(stats.received, 0);
    assert_eq!(stats.processed, 0);
    assert_eq!(stats.skipped, 0);
    assert_eq!(stats.failed, 0);
    assert_eq!(sink.count(), 0);
}

#[tokio::test]
async fn pipeline_ingest_batch_single_message() {
    let sink = CountingSink::new();
    let pipeline = PipelineBuilder::new()
        .with_processor(AlwaysProcessProcessor)
        .with_sink(sink.clone())
        .build();

    let message = test_message();
    let messages = vec![Box::new(message) as Box<dyn eidoscope_ingestor::message::Message>];
    let stats = pipeline.ingest_batch(messages).await;

    assert_eq!(stats.received, 1);
    assert_eq!(stats.processed, 1);
    assert_eq!(stats.skipped, 0);
    assert_eq!(stats.failed, 0);
    assert_eq!(sink.count(), 1);
}

#[tokio::test]
async fn pipeline_ingest_batch_multiple_processed() {
    let sink = CountingSink::new();
    let pipeline = PipelineBuilder::new()
        .with_processor(AlwaysProcessProcessor)
        .with_sink(sink.clone())
        .build();

    let source = test_source();
    let messages: Vec<Box<dyn eidoscope_ingestor::message::Message>> = (0..10)
        .map(|i| {
            let payload = format!("message {}", i).into_bytes();
            Box::new(eidoscope_ingestor::message::GenericMessage::new(
                payload,
                source.clone(),
            )) as Box<dyn eidoscope_ingestor::message::Message>
        })
        .collect();

    let stats = pipeline.ingest_batch(messages).await;

    assert_eq!(stats.received, 10);
    assert_eq!(stats.processed, 10);
    assert_eq!(stats.skipped, 0);
    assert_eq!(stats.failed, 0);
    assert_eq!(sink.count(), 10);
}

#[tokio::test]
async fn pipeline_ingest_batch_all_skipped() {
    let sink = CountingSink::new();
    let processor = AlwaysSkipProcessor::new(SkipReason::Filtered {
        reason: "test filter".to_string(),
    });
    let pipeline = PipelineBuilder::new()
        .with_processor(processor)
        .with_sink(sink.clone())
        .build();

    let source = test_source();
    let messages: Vec<Box<dyn eidoscope_ingestor::message::Message>> = (0..5)
        .map(|i| {
            let payload = format!("message {}", i).into_bytes();
            Box::new(eidoscope_ingestor::message::GenericMessage::new(
                payload,
                source.clone(),
            )) as Box<dyn eidoscope_ingestor::message::Message>
        })
        .collect();

    let stats = pipeline.ingest_batch(messages).await;

    assert_eq!(stats.received, 5);
    assert_eq!(stats.processed, 0);
    assert_eq!(stats.skipped, 5);
    assert_eq!(stats.failed, 0);
    assert_eq!(sink.count(), 0);
}

#[tokio::test]
async fn pipeline_ingest_batch_continues_after_failure() {
    let sink = FailingSink::new(SinkError::Unavailable {
        sink_name: "test".to_string(),
    });
    let pipeline = PipelineBuilder::new()
        .with_processor(AlwaysProcessProcessor)
        .with_sink(sink)
        .build();

    let source = test_source();
    let messages: Vec<Box<dyn eidoscope_ingestor::message::Message>> = (0..5)
        .map(|i| {
            let payload = format!("message {}", i).into_bytes();
            Box::new(eidoscope_ingestor::message::GenericMessage::new(
                payload,
                source.clone(),
            )) as Box<dyn eidoscope_ingestor::message::Message>
        })
        .collect();

    let stats = pipeline.ingest_batch(messages).await;

    assert_eq!(stats.received, 5);
    assert_eq!(stats.processed, 0);
    assert_eq!(stats.skipped, 0);
    assert_eq!(stats.failed, 5);
}

#[tokio::test]
async fn pipeline_flush_calls_sink_flush() {
    let sink = CountingSink::new();
    let pipeline = PipelineBuilder::new()
        .with_processor(AlwaysProcessProcessor)
        .with_sink(sink)
        .build();

    let result = pipeline.flush().await;
    assert!(result.is_ok());
}
