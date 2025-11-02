//! Builder pattern for constructing pipelines.

use super::Pipeline;
use crate::{processor::Processor, sink::Sink};

/// Builder for constructing ingestion pipelines.
///
/// This implements the builder pattern with compile-time guarantees that
/// all required components (processor and sink) are provided before a
/// pipeline can be built.
///
/// # Type State Pattern
///
/// The builder uses phantom types to track which components have been set:
/// - `NoProcessor` -> `HasProcessor<P>` when processor is set
/// - `NoSink` -> `HasSink<S>` when sink is set
///
/// Only when both are set can `build()` be called.
///
/// # Examples
///
/// ```
/// use eidoscope_ingestor::{
///     pipeline::PipelineBuilder,
///     processor::PassthroughProcessor,
///     sink::TracingSink,
/// };
///
/// let pipeline = PipelineBuilder::new()
///     .with_processor(PassthroughProcessor)
///     .with_sink(TracingSink::new())
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct PipelineBuilder<P = NoProcessor, S = NoSink> {
    processor: P,
    sink: S,
}

/// Type-level marker indicating no processor has been set.
#[derive(Debug, Default)]
pub struct NoProcessor;

/// Type-level marker indicating no sink has been set.
#[derive(Debug, Default)]
pub struct NoSink;

impl PipelineBuilder<NoProcessor, NoSink> {
    /// Creates a new pipeline builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::pipeline::PipelineBuilder;
    ///
    /// let builder = PipelineBuilder::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            processor: NoProcessor,
            sink: NoSink,
        }
    }
}

impl<S> PipelineBuilder<NoProcessor, S> {
    /// Sets the processor for the pipeline.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::{
    ///     pipeline::PipelineBuilder,
    ///     processor::PassthroughProcessor,
    /// };
    ///
    /// let builder = PipelineBuilder::new()
    ///     .with_processor(PassthroughProcessor);
    /// ```
    #[must_use]
    pub fn with_processor<P>(self, processor: P) -> PipelineBuilder<P, S>
    where
        P: Processor,
    {
        PipelineBuilder {
            processor,
            sink: self.sink,
        }
    }
}

impl<P> PipelineBuilder<P, NoSink> {
    /// Sets the sink for the pipeline.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::{
    ///     pipeline::PipelineBuilder,
    ///     processor::PassthroughProcessor,
    ///     sink::TracingSink,
    /// };
    ///
    /// let builder = PipelineBuilder::new()
    ///     .with_processor(PassthroughProcessor)
    ///     .with_sink(TracingSink::new());
    /// ```
    #[must_use]
    pub fn with_sink<S>(self, sink: S) -> PipelineBuilder<P, S>
    where
        S: Sink,
    {
        PipelineBuilder {
            processor: self.processor,
            sink,
        }
    }
}

impl<P, S> PipelineBuilder<P, S>
where
    P: Processor,
    S: Sink,
{
    /// Builds the pipeline.
    ///
    /// This method is only available when both processor and sink have been set,
    /// ensuring that incomplete pipelines cannot be constructed.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::{
    ///     pipeline::PipelineBuilder,
    ///     processor::PassthroughProcessor,
    ///     sink::TracingSink,
    /// };
    ///
    /// let pipeline = PipelineBuilder::new()
    ///     .with_processor(PassthroughProcessor)
    ///     .with_sink(TracingSink::new())
    ///     .build();
    /// ```
    #[must_use]
    pub fn build(self) -> Pipeline<P, S> {
        Pipeline::new(self.processor, self.sink)
    }
}
