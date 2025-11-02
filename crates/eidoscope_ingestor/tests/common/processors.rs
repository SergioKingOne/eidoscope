//! Test processor implementations.

use eidoscope_ingestor::{
    error::{IngestorError, ProcessingError, Result},
    message::Message,
    processor::{ProcessingDecision, Processor, SkipReason},
};

/// A processor that always returns Process.
#[derive(Debug, Clone, Copy, Default)]
pub struct AlwaysProcessProcessor;

impl Processor for AlwaysProcessProcessor {
    async fn process(&self, _message: &dyn Message) -> Result<ProcessingDecision> {
        Ok(ProcessingDecision::Process)
    }
}

/// A processor that always skips messages.
#[derive(Debug, Clone)]
pub struct AlwaysSkipProcessor {
    pub reason: SkipReason,
}

impl AlwaysSkipProcessor {
    pub fn new(reason: SkipReason) -> Self {
        Self { reason }
    }
}

impl Processor for AlwaysSkipProcessor {
    async fn process(&self, _message: &dyn Message) -> Result<ProcessingDecision> {
        Ok(ProcessingDecision::Skip(self.reason.clone()))
    }
}

/// A processor that fails with an error.
#[derive(Debug, Clone)]
pub struct FailingProcessor {
    pub error: ProcessingError,
}

impl FailingProcessor {
    pub fn new(error: ProcessingError) -> Self {
        Self { error }
    }
}

impl Processor for FailingProcessor {
    async fn process(&self, _message: &dyn Message) -> Result<ProcessingDecision> {
        Err(IngestorError::Processing(self.error.clone()))
    }
}
