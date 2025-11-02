//! Test sink implementations.

use eidoscope_ingestor::{
    error::{IngestorError, Result, SinkError},
    message::Message,
    sink::Sink,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

/// A sink that counts how many messages it has received.
#[derive(Debug, Clone, Default)]
pub struct CountingSink {
    count: Arc<AtomicUsize>,
}

impl CountingSink {
    pub fn new() -> Self {
        Self {
            count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn count(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }
}

impl Sink for CountingSink {
    async fn send(&self, _message: Box<dyn Message>) -> Result<()> {
        self.count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn name(&self) -> &str {
        "CountingSink"
    }
}

/// A sink that collects all messages it receives.
#[derive(Debug, Clone, Default)]
pub struct CollectingSink {
    messages: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl CollectingSink {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn messages(&self) -> Vec<Vec<u8>> {
        self.messages.lock().unwrap().clone()
    }
}

impl Sink for CollectingSink {
    async fn send(&self, message: Box<dyn Message>) -> Result<()> {
        self.messages
            .lock()
            .unwrap()
            .push(message.payload().to_vec());
        Ok(())
    }

    fn name(&self) -> &str {
        "CollectingSink"
    }
}

/// A sink that always fails.
#[derive(Debug, Clone)]
pub struct FailingSink {
    pub error: SinkError,
}

impl FailingSink {
    pub fn new(error: SinkError) -> Self {
        Self { error }
    }
}

impl Sink for FailingSink {
    async fn send(&self, _message: Box<dyn Message>) -> Result<()> {
        Err(IngestorError::Sink(self.error.clone()))
    }

    fn name(&self) -> &str {
        "FailingSink"
    }
}
