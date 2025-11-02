//! Tests for ProcessingDecision and SkipReason.

use eidoscope_ingestor::processor::{ProcessingDecision, SkipReason};

#[test]
fn processing_decision_process_variant() {
    let decision = ProcessingDecision::Process;
    assert!(decision.should_process());
    assert!(!decision.should_skip());
    assert!(decision.skip_reason().is_none());
}

#[test]
fn processing_decision_skip_variant() {
    let decision = ProcessingDecision::Skip(SkipReason::Duplicate);
    assert!(!decision.should_process());
    assert!(decision.should_skip());
    assert!(decision.skip_reason().is_some());
}

#[test]
fn processing_decision_skip_reason_returns_correct_value() {
    let reason = SkipReason::RateLimited;
    let decision = ProcessingDecision::Skip(reason.clone());

    assert_eq!(decision.skip_reason(), Some(&reason));
}

#[test]
fn skip_reason_duplicate_display() {
    let reason = SkipReason::Duplicate;
    assert_eq!(reason.to_string(), "Duplicate message");
}

#[test]
fn skip_reason_filtered_helper() {
    let reason = SkipReason::filtered("too old");
    assert_eq!(reason.to_string(), "Filtered: too old");
}

#[test]
fn skip_reason_invalid_helper() {
    let reason = SkipReason::invalid("malformed JSON");
    assert_eq!(reason.to_string(), "Invalid: malformed JSON");
}

#[test]
fn skip_reason_out_of_window_helper() {
    let reason = SkipReason::out_of_window("event time outside processing window");
    assert_eq!(
        reason.to_string(),
        "Out of window: event time outside processing window"
    );
}

#[test]
fn skip_reason_rate_limited_display() {
    let reason = SkipReason::RateLimited;
    assert_eq!(reason.to_string(), "Rate limited");
}

#[test]
fn skip_reason_custom_helper() {
    let reason = SkipReason::custom("business_rule", "customer not active");
    assert_eq!(reason.to_string(), "business_rule: customer not active");
}

