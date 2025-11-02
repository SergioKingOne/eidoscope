//! Stream processing for Eidoscope message ingestion
//!
//! This crate provides abstractions and implementations for consuming messages from
//! various streaming sources (e.g., Kafka, Redis Streams, in-memory channels) and
//! processing them through the Eidoscope ingestion pipeline.
//!
//! # Architecture
//!
//! The stream processor follows a layered architecture:
//!
//! 1. **Stream Consumer**: Abstracts message consumption from different sources
//! 2. **Stream Processor**: Orchestrates consumption and processing through pipelines
//! 3. **Offset Management**: Tracks processing progress for resumability
//!
//! # Example
//!
//! ```rust,no_run
//! use eidoscope_ingestor::{
//!     message::GenericMessage,
//!     processor::PassthroughProcessor,
//!     sink::TracingSink,
//!     pipeline::PipelineBuilder,
//! };
//! use eidoscope_stream_processor::{
//!     consumer::channel::ChannelConsumer,
//!     StreamProcessor,
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a channel consumer (for testing/examples)
//! let (tx, rx) = tokio::sync::mpsc::channel(100);
//! let consumer = ChannelConsumer::new(rx);
//!
//! // Build the ingestion pipeline
//! let pipeline = PipelineBuilder::new()
//!     .with_processor(PassthroughProcessor)
//!     .with_sink(TracingSink::new(tracing::Level::INFO))
//!     .build()?;
//!
//! // Create the stream processor
//! let mut processor = StreamProcessor::new(consumer, pipeline);
//!
//! // Process messages from the stream
//! processor.run().await?;
//! # Ok(())
//! # }
//! ```

pub mod consumer;
pub mod error;
mod processor;

pub use error::{Result, StreamError};
pub use processor::{StreamProcessor, StreamStats};
