//! Comprehensive End-to-End Integration Example
//!
//! This example demonstrates a realistic log ingestion pipeline using eidoscope_ingestor.
//!
//! The scenario: An application monitoring system that ingests JSON log events from
//! multiple microservices, validates them, filters based on business rules, and stores
//! valid logs for analysis.
//!
//! This demonstrates:
//! - Custom message types (structured log events)
//! - Custom processors (validation + filtering)
//! - Custom sinks (log storage)
//! - Pipeline construction and execution
//! - Batch processing with statistics
//! - Error handling in production scenarios
//!
//! Run with: cargo run --example comprehensive_integration

use eidoscope_ingestor::{
    error::Result,
    message::{Message, MessageId, MessageMetadata, SourceName, Timestamp},
    pipeline::PipelineBuilder,
    processor::{ProcessingDecision, Processor, SkipReason},
    sink::Sink,
};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

// ============================================================================
// CUSTOM MESSAGE TYPE
// ============================================================================

/// A log event message containing structured JSON data from microservices.
#[derive(Debug)]
struct LogEvent {
    id: MessageId,
    metadata: MessageMetadata,
    json_payload: Vec<u8>,
}

impl LogEvent {
    fn new(json: &str, source: SourceName) -> Self {
        Self {
            id: MessageId::new_or_generate(""),
            metadata: MessageMetadata::new(source),
            json_payload: json.as_bytes().to_vec(),
        }
    }

    fn with_timestamp(json: &str, source: SourceName, timestamp: Timestamp) -> Self {
        Self {
            id: MessageId::new_or_generate(""),
            metadata: MessageMetadata::with_timestamp(timestamp, source),
            json_payload: json.as_bytes().to_vec(),
        }
    }
}

impl Message for LogEvent {
    fn id(&self) -> &MessageId {
        &self.id
    }

    fn metadata(&self) -> &MessageMetadata {
        &self.metadata
    }

    fn payload(&self) -> &[u8] {
        &self.json_payload
    }

    fn content_type(&self) -> Option<&str> {
        Some("application/json")
    }
}

// ============================================================================
// CUSTOM PROCESSOR
// ============================================================================

/// A processor that validates log events and enforces business rules.
///
/// This simulates a production log ingestion pipeline that:
/// - Validates JSON structure
/// - Enforces size limits to prevent memory issues
/// - Filters out stale logs beyond retention window
struct LogEventProcessor {
    max_size: usize,
    retention_window_seconds: u64,
}

impl LogEventProcessor {
    fn new(max_size: usize, retention_window_seconds: u64) -> Self {
        Self {
            max_size,
            retention_window_seconds,
        }
    }

    fn is_valid_json(payload: &[u8]) -> bool {
        serde_json::from_slice::<serde_json::Value>(payload).is_ok()
    }
}

impl Processor for LogEventProcessor {
    async fn process(&self, message: &dyn Message) -> Result<ProcessingDecision> {
        // Validate JSON structure
        if !Self::is_valid_json(message.payload()) {
            return Ok(ProcessingDecision::Skip(SkipReason::invalid(
                "Invalid JSON structure",
            )));
        }

        // Enforce size limit to prevent memory issues
        if message.size() > self.max_size {
            return Ok(ProcessingDecision::Skip(SkipReason::filtered(format!(
                "Log event size {} bytes exceeds limit of {} bytes",
                message.size(),
                self.max_size
            ))));
        }

        // Filter out logs beyond retention window
        let now = SystemTime::now();
        let event_time = message.metadata().timestamp.as_system_time();
        let age = now
            .duration_since(event_time)
            .unwrap_or(Duration::from_secs(0));

        if age.as_secs() > self.retention_window_seconds {
            return Ok(ProcessingDecision::Skip(SkipReason::out_of_window(
                format!(
                    "Log event is {} seconds old, beyond retention window",
                    age.as_secs()
                ),
            )));
        }

        // All validation passed - accept the log event
        Ok(ProcessingDecision::Process)
    }
}

// ============================================================================
// CUSTOM SINK
// ============================================================================

/// A sink that stores log events (simulating a log storage backend).
///
/// In a real system, this would write to a database, object storage, or
/// log aggregation service. Here we store in memory for demonstration.
#[derive(Clone)]
struct LogStorageSink {
    stored_logs: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl LogStorageSink {
    fn new() -> Self {
        Self {
            stored_logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn count(&self) -> usize {
        self.stored_logs.lock().unwrap().len()
    }

    fn get_stored_logs(&self) -> Vec<Vec<u8>> {
        self.stored_logs.lock().unwrap().clone()
    }
}

impl Sink for LogStorageSink {
    async fn send(&self, message: Box<dyn Message>) -> Result<()> {
        // Simulate async I/O delay
        tokio::time::sleep(Duration::from_millis(1)).await;

        let payload = message.payload().to_vec();
        self.stored_logs.lock().unwrap().push(payload);
        Ok(())
    }

    async fn flush(&self) -> Result<()> {
        // In production, this would flush write buffers to storage
        Ok(())
    }

    fn name(&self) -> &str {
        "LogStorageSink"
    }
}

// ============================================================================
// MAIN: UNIFIED LOG INGESTION EXAMPLE
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════════════════╗");
    println!("║    Eidoscope Ingestor: Log Ingestion Pipeline Example                 ║");
    println!("╚════════════════════════════════════════════════════════════════════════╝\n");

    // ========================================================================
    // STEP 1: Set up the log ingestion pipeline
    // ========================================================================
    println!("📦 Setting up log ingestion pipeline...");
    println!("   - Processor: Validates JSON, enforces size limit (10KB), 5-minute retention");
    println!("   - Sink: In-memory log storage (simulates database)\n");

    let log_storage = LogStorageSink::new();
    let processor = LogEventProcessor::new(10_240, 300); // 10KB max, 5 min retention
    let pipeline = PipelineBuilder::new()
        .with_processor(processor)
        .with_sink(log_storage.clone())
        .build();

    println!("✅ Pipeline ready\n");

    // ========================================================================
    // STEP 2: Ingest logs from various microservices
    // ========================================================================
    println!("📨 Ingesting log events from microservices...\n");

    // Create some realistic log events from different services
    let auth_service = SourceName::new("auth-service").unwrap();
    let api_gateway = SourceName::new("api-gateway").unwrap();
    let payment_service = SourceName::new("payment-service").unwrap();

    let logs: Vec<Box<dyn Message>> = vec![
        // Valid logs from different services
        Box::new(LogEvent::new(
            r#"{"level":"info","user_id":12345,"action":"login","success":true}"#,
            auth_service.clone(),
        )),
        Box::new(LogEvent::new(
            r#"{"level":"info","endpoint":"/api/users","method":"GET","status":200}"#,
            api_gateway.clone(),
        )),
        Box::new(LogEvent::new(
            r#"{"level":"info","transaction_id":"tx_789","amount":99.99,"currency":"USD"}"#,
            payment_service.clone(),
        )),
        // This log has invalid JSON - will be skipped
        Box::new(LogEvent::new(
            r#"{"level":"error","incomplete json"#,
            api_gateway.clone(),
        )),
        // This log is too large - will be filtered
        Box::new(LogEvent::new(
            &format!(
                r#"{{"level":"debug","large_data":"{}"}}"#,
                "x".repeat(12_000)
            ),
            auth_service.clone(),
        )),
        // This log is from 10 minutes ago - beyond retention window
        Box::new({
            let old_time = SystemTime::now() - Duration::from_secs(600);
            let timestamp = Timestamp::from_system_time(old_time).unwrap();
            LogEvent::with_timestamp(
                r#"{"level":"warn","msg":"stale log"}"#,
                api_gateway.clone(),
                timestamp,
            )
        }),
        // More valid logs
        Box::new(LogEvent::new(
            r#"{"level":"warn","user_id":12345,"failed_attempts":3}"#,
            auth_service.clone(),
        )),
        Box::new(LogEvent::new(
            r#"{"level":"info","endpoint":"/api/health","method":"GET","status":200}"#,
            api_gateway,
        )),
    ];

    // ========================================================================
    // STEP 3: Process the batch and collect statistics
    // ========================================================================
    let stats = pipeline.ingest_batch(logs).await;

    println!("📊 Ingestion Statistics:");
    println!("   ├─ Total received:    {}", stats.received);
    println!("   ├─ Successfully stored: {}", stats.processed);
    println!("   ├─ Skipped/filtered:  {}", stats.skipped);
    println!("   └─ Failed:            {}\n", stats.failed);

    // Verify the results match expectations
    assert_eq!(
        stats.received, 8,
        "Should have received 8 log events total"
    );
    assert_eq!(
        stats.processed, 5,
        "Should have processed 5 valid log events"
    );
    assert_eq!(
        stats.skipped, 3,
        "Should have skipped 3 log events (invalid JSON, too large, too old)"
    );
    assert_eq!(stats.failed, 0, "Should have 0 failures");

    // ========================================================================
    // STEP 4: Verify stored logs
    // ========================================================================
    println!("💾 Verifying log storage...");
    let stored = log_storage.get_stored_logs();
    println!("   Total logs in storage: {}", stored.len());
    println!("\n   Sample stored logs:");

    for (i, log) in stored.iter().take(3).enumerate() {
        let log_str = String::from_utf8_lossy(log);
        println!("   {}. {}", i + 1, log_str);
    }

    assert_eq!(
        stored.len(),
        5,
        "Storage should contain exactly 5 valid logs"
    );

    // ========================================================================
    // STEP 5: Demonstrate individual log ingestion with error visibility
    // ========================================================================
    println!("\n📝 Processing individual log events with full error visibility...\n");

    // Process a valid log
    let valid_log = LogEvent::new(
        r#"{"level":"info","msg":"individual event test"}"#,
        SourceName::new("test-service").unwrap(),
    );
    let decision = pipeline.ingest(Box::new(valid_log)).await?;
    println!("   ✅ Valid log -> {:?}", decision);

    // Process an invalid log - shows the skip reason
    let invalid_log = LogEvent::new(
        "{this is not valid json}",
        SourceName::new("test-service").unwrap(),
    );
    let decision = pipeline.ingest(Box::new(invalid_log)).await?;
    if let ProcessingDecision::Skip(reason) = decision {
        println!("   ⏭  Invalid JSON -> Skipped: {:?}", reason);
    }

    // ========================================================================
    // STEP 6: Flush and finalize
    // ========================================================================
    println!("\n🔄 Flushing pipeline...");
    pipeline.flush().await?;
    println!("   ✅ Flush complete\n");

    // ========================================================================
    // SUMMARY
    // ========================================================================
    println!("╔════════════════════════════════════════════════════════════════════════╗");
    println!("║                           Summary                                      ║");
    println!("╠════════════════════════════════════════════════════════════════════════╣");
    println!("║  ✅  Log ingestion pipeline successfully processed events              ║");
    println!("║  ✅  Custom LogEvent message type demonstrated                         ║");
    println!("║  ✅  Custom LogEventProcessor with validation & filtering              ║");
    println!("║  ✅  Custom LogStorageSink with async I/O simulation                   ║");
    println!("║  ✅  Batch processing with comprehensive statistics                    ║");
    println!("║  ✅  Individual message processing with error visibility               ║");
    println!("║  ✅  Pipeline flush behavior                                           ║");
    println!("╚════════════════════════════════════════════════════════════════════════╝\n");

    println!("Final storage count: {} logs", log_storage.count());
    println!("\n🎉 Example completed successfully!");

    Ok(())
}
