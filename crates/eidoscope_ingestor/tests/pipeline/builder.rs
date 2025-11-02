//! Tests for PipelineBuilder.

use crate::common::{
    fixtures::test_message, processors::AlwaysProcessProcessor, sinks::CountingSink,
};
use eidoscope_ingestor::{pipeline::PipelineBuilder, sink::Sink};

#[test]
fn pipeline_builder_processor_then_sink() {
    let sink = CountingSink::new();
    let pipeline = PipelineBuilder::new()
        .with_processor(AlwaysProcessProcessor)
        .with_sink(sink)
        .build();

    assert_eq!(pipeline.sink().name(), "CountingSink");
}

#[test]
fn pipeline_builder_sink_then_processor() {
    let sink = CountingSink::new();
    // Can set sink first, then processor
    let pipeline = PipelineBuilder::new()
        .with_sink(sink)
        .with_processor(AlwaysProcessProcessor)
        .build();

    assert_eq!(pipeline.sink().name(), "CountingSink");
}

#[tokio::test]
async fn pipeline_builder_creates_working_pipeline() {
    let sink = CountingSink::new();
    let pipeline = PipelineBuilder::new()
        .with_processor(AlwaysProcessProcessor)
        .with_sink(sink.clone())
        .build();

    let message = test_message();
    pipeline.ingest(Box::new(message)).await.unwrap();

    assert_eq!(sink.count(), 1);
}

