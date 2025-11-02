# eidoscope_stream_processor

Stream processing layer for Eidoscope message ingestion.

## Overview

This crate provides abstractions and implementations for consuming messages from various streaming sources and processing them through Eidoscope ingestion pipelines.

## Features

- **Stream Consumer Abstraction**: Trait-based design supporting multiple stream sources
- **Built-in Channel Consumer**: In-memory channel implementation for testing and development
- **Offset Management**: Track processing progress for resumability
- **Statistics Tracking**: Monitor consumption, processing, and error rates
- **Configurable Behavior**: Auto-commit, failure thresholds, stats intervals
- **Graceful Shutdown**: Proper cleanup and resource management

## Architecture

```
StreamConsumer (trait)
    │
    ├─ consume() -> ConsumedMessage
    ├─ commit(offset) -> Result<()>
    └─ close() -> Result<()>
    │
    └─ Implementations:
       └─ ChannelConsumer (in-memory, for testing)

StreamProcessor<C, P, S>
    │
    ├─ Combines: StreamConsumer + Pipeline
    ├─ run() - Continuous processing loop
    ├─ process_one() - Single message processing
    └─ stats() - Access processing statistics
```

## Usage

### Basic Example

```rust
use eidoscope_ingestor::{
    processor::PassthroughProcessor,
    sink::TracingSink,
    pipeline::PipelineBuilder,
};
use eidoscope_stream_processor::{
    consumer::channel::ChannelConsumerBuilder,
    StreamProcessor,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a channel consumer
    let (consumer, sender) = ChannelConsumerBuilder::new()
        .with_capacity(100)
        .build();

    // Build the ingestion pipeline
    let pipeline = PipelineBuilder::new()
        .with_processor(PassthroughProcessor)
        .with_sink(TracingSink::new(tracing::Level::INFO))
        .build()?;

    // Create and run the stream processor
    let mut processor = StreamProcessor::new(consumer, pipeline);

    // Spawn a producer to send messages
    tokio::spawn(async move {
        // Send messages to sender...
    });

    let stats = processor.run().await?;
    println!("Processed {} messages", stats.processed);

    Ok(())
}
```

### Custom Stream Consumer

Implement the `StreamConsumer` trait for your streaming source:

```rust
use eidoscope_stream_processor::consumer::{StreamConsumer, ConsumedMessage};
use eidoscope_stream_processor::error::Result;

struct KafkaConsumer {
    // Kafka-specific fields
}

impl StreamConsumer for KafkaConsumer {
    async fn consume(&mut self) -> Result<Option<ConsumedMessage>> {
        // Consume from Kafka
        todo!()
    }

    async fn commit(&mut self, offset: &str) -> Result<()> {
        // Commit Kafka offset
        todo!()
    }

    fn source_name(&self) -> &str {
        "kafka"
    }
}
```

### Configuration

```rust
use eidoscope_stream_processor::StreamConfig;
use std::time::Duration;

let config = StreamConfig::new()
    .with_auto_commit(true)
    .with_stats_interval(Some(Duration::from_secs(60)))
    .with_max_consecutive_failures(Some(10));

let processor = StreamProcessor::with_config(consumer, pipeline, config);
```

## Statistics

The `StreamStats` struct tracks:
- `consumed`: Total messages consumed from the stream
- `processed`: Messages successfully processed
- `skipped`: Messages filtered by the processor
- `failed`: Messages that encountered errors
- `committed`: Offsets successfully committed
- `commit_failures`: Failed offset commits

## Examples

Run the comprehensive example:

```bash
cargo run --example end_to_end_stream
```

## Testing

```bash
cargo test --package eidoscope_stream_processor
```

## Future Implementations

Potential stream consumer implementations:
- Kafka consumer (using `rdkafka`)
- Redis Streams consumer
- AWS Kinesis consumer
- NATS JetStream consumer
- File-based consumer (tail -f style)

## License

See workspace license.
