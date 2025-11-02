//! # Eidoscope Ingestor
//!
//! A type-safe, runtime-agnostic message ingestion library.
//!
//! ## Overview
//!
//! The Eidoscope Ingestor provides a composable pipeline for receiving, processing,
//! and sinking messages. It applies type-driven design principles to make illegal
//! states unrepresentable and provide compile-time guarantees about pipeline correctness.
//!
//! ## Core Concepts
//!
//! - **Messages**: Data flowing through the pipeline ([`message`] module)
//! - **Processors**: Decision logic for filtering/transforming ([`processor`] module)
//! - **Sinks**: Output destinations for processed messages ([`sink`] module)
//! - **Pipelines**: Orchestration of the complete flow ([`pipeline`] module)
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────┐     ┌───────────┐     ┌──────┐
//! │ Message │ --> │ Processor │ --> │ Sink │
//! └─────────┘     └───────────┘     └──────┘
//!                       │
//!                       v
//!               ProcessingDecision
//!                  / Process \ Skip
//! ```
//!
//! ## Quick Start
//!
//! ```
//! use eidoscope_ingestor::{
//!     message::{GenericMessage, SourceName},
//!     pipeline::PipelineBuilder,
//!     processor::PassthroughProcessor,
//!     sink::TracingSink,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a simple pipeline
//! let pipeline = PipelineBuilder::new()
//!     .with_processor(PassthroughProcessor)
//!     .with_sink(TracingSink::new())
//!     .build();
//!
//! // Ingest a message
//! let source = SourceName::new("my-app")?;
//! let message = GenericMessage::new(b"Hello, world!".to_vec(), source);
//! pipeline.ingest(Box::new(message))?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Type-Driven Design
//!
//! This library applies principles from functional programming to create
//! a robust, self-documenting API:
//!
//! ### Validated Construction
//!
//! Types like [`MessageId`](message::MessageId) and [`SourceName`](message::SourceName)
//! use validated constructors that return `Result`, ensuring invalid values
//! cannot exist in the system.
//!
//! ```
//! use eidoscope_ingestor::message::MessageId;
//!
//! // Valid construction
//! let id = MessageId::new("msg-123")?;
//!
//! // Invalid construction fails at creation
//! assert!(MessageId::new("").is_err());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Making Illegal States Unrepresentable
//!
//! The [`ProcessingDecision`](processor::ProcessingDecision) type uses a discriminated
//! union to ensure skipped messages always have a reason:
//!
//! ```
//! use eidoscope_ingestor::processor::{ProcessingDecision, SkipReason};
//!
//! // Can't create a skip without a reason - it's impossible!
//! let decision = ProcessingDecision::Skip(SkipReason::Duplicate);
//! ```
//!
//! ### Builder Pattern with Type States
//!
//! The [`PipelineBuilder`](pipeline::PipelineBuilder) uses phantom types to ensure
//! pipelines are always constructed with both processor and sink:
//!
//! ```compile_fail
//! use eidoscope_ingestor::pipeline::PipelineBuilder;
//!
//! // This won't compile - missing sink!
//! let pipeline = PipelineBuilder::new()
//!     .with_processor(PassthroughProcessor)
//!     .build();
//! ```
//!
//! ## Features
//!
//! - `async` (default): Enables async variants of traits and types
//!
//! ## Runtime Agnostic
//!
//! The library provides both synchronous and asynchronous APIs, allowing
//! use in any context without forcing a specific runtime.
//!
//! Synchronous:
//! ```
//! use eidoscope_ingestor::processor::Processor;
//! # use eidoscope_ingestor::processor::PassthroughProcessor;
//! # use eidoscope_ingestor::message::{GenericMessage, SourceName};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let processor = PassthroughProcessor;
//! # let source = SourceName::new("test")?;
//! # let message = GenericMessage::new(b"data".to_vec(), source);
//! let decision = processor.process(&message)?;
//! # Ok(())
//! # }
//! ```
//!
//! Asynchronous (with `async` feature):
//! ```
//! # #[cfg(feature = "async")]
//! # {
//! use eidoscope_ingestor::processor::AsyncProcessor;
//! # use eidoscope_ingestor::processor::PassthroughProcessor;
//! # use eidoscope_ingestor::message::{GenericMessage, SourceName};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let processor = PassthroughProcessor;
//! # let source = SourceName::new("test")?;
//! # let message = GenericMessage::new(b"data".to_vec(), source);
//! let decision = processor.process(&message).await?;
//! # Ok(())
//! # }
//! # }
//! ```

#![warn(missing_docs)]
#![forbid(unsafe_code)]

pub mod error;
pub mod message;
pub mod pipeline;
pub mod processor;
pub mod sink;

// Re-export key types at the crate root for convenience
pub use error::{IngestorError, Result};
