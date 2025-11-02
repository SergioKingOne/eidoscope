//! Tests for processor types.

mod decision;
mod passthrough;

use crate::common::{
    fixtures::test_message,
    processors::{AlwaysProcessProcessor, AlwaysSkipProcessor, FailingProcessor},
};
use eidoscope_ingestor::{
    error::{IngestorError, ProcessingError},
    processor::{ProcessingDecision, Processor, SkipReason},
};

// ===== Processor Trait Tests =====

#[tokio::test]
async fn processor_trait_always_process() {
    let processor = AlwaysProcessProcessor;
    let message = test_message();

    let decision = processor.process(&message).await.unwrap();

    assert!(matches!(decision, ProcessingDecision::Process));
    assert!(decision.should_process());
}

#[tokio::test]
async fn processor_trait_always_skip() {
    let processor = AlwaysSkipProcessor::new(SkipReason::Duplicate);
    let message = test_message();

    let decision = processor.process(&message).await.unwrap();

    assert!(matches!(decision, ProcessingDecision::Skip(_)));
    assert!(decision.should_skip());
}

#[tokio::test]
async fn processor_trait_error_propagation() {
    let processor = FailingProcessor::new(ProcessingError::Failed {
        reason: "test failure".to_string(),
    });
    let message = test_message();

    let result = processor.process(&message).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), IngestorError::Processing(_)));
}

#[tokio::test]
async fn processor_trait_with_different_skip_reasons() {
    let skip_reasons = vec![
        SkipReason::Duplicate,
        SkipReason::RateLimited,
        SkipReason::filtered("test filter"),
        SkipReason::invalid("test invalid"),
        SkipReason::custom("test", "custom reason"),
    ];

    for reason in skip_reasons {
        let processor = AlwaysSkipProcessor::new(reason.clone());
        let message = test_message();

        let decision = processor.process(&message).await.unwrap();

        assert!(decision.should_skip());
        assert_eq!(decision.skip_reason(), Some(&reason));
    }
}

