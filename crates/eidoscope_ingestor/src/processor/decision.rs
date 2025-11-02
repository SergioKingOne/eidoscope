//! Processing decision types.
//!
//! This module implements the "making illegal states unrepresentable" pattern
//! by using sum types to explicitly model all possible processing outcomes.

use serde::{Deserialize, Serialize};
use std::fmt;

/// The outcome of processing a message.
///
/// This type makes explicit whether a message should continue through the pipeline
/// or be skipped. By using a discriminated union instead of a boolean or optional,
/// we ensure that skip decisions always include a reason.
///
/// # Design Rationale
///
/// Rather than using `bool` or `Option`, this type encodes business logic directly:
/// - A message is either processed (continues to sink) or skipped (with reason)
/// - Skipped messages **must** have a reason, making debugging and auditing easier
/// - Pattern matching forces explicit handling of both cases
///
/// # Examples
///
/// ```
/// use eidoscope_ingestor::processor::{ProcessingDecision, SkipReason};
///
/// let decision = ProcessingDecision::Process;
/// assert!(decision.should_process());
///
/// let decision = ProcessingDecision::Skip(SkipReason::Filtered {
///     reason: "Rate limit exceeded".to_string(),
/// });
/// assert!(!decision.should_process());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "decision", content = "reason")]
pub enum ProcessingDecision {
    /// The message should continue through the pipeline to the sink.
    Process,

    /// The message should be skipped and not sent to the sink.
    ///
    /// The associated [`SkipReason`] provides context for why the message was skipped.
    Skip(SkipReason),
}

impl ProcessingDecision {
    /// Returns `true` if the decision is to process the message.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::processor::ProcessingDecision;
    ///
    /// let decision = ProcessingDecision::Process;
    /// assert!(decision.should_process());
    /// ```
    #[must_use]
    pub const fn should_process(&self) -> bool {
        matches!(self, Self::Process)
    }

    /// Returns `true` if the decision is to skip the message.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::processor::{ProcessingDecision, SkipReason};
    ///
    /// let decision = ProcessingDecision::Skip(SkipReason::Duplicate);
    /// assert!(decision.should_skip());
    /// ```
    #[must_use]
    pub const fn should_skip(&self) -> bool {
        matches!(self, Self::Skip(_))
    }

    /// Returns the skip reason if the message should be skipped.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::processor::{ProcessingDecision, SkipReason};
    ///
    /// let decision = ProcessingDecision::Skip(SkipReason::Duplicate);
    /// assert!(decision.skip_reason().is_some());
    ///
    /// let decision = ProcessingDecision::Process;
    /// assert!(decision.skip_reason().is_none());
    /// ```
    #[must_use]
    pub const fn skip_reason(&self) -> Option<&SkipReason> {
        match self {
            Self::Process => None,
            Self::Skip(reason) => Some(reason),
        }
    }
}

impl fmt::Display for ProcessingDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Process => write!(f, "Process"),
            Self::Skip(reason) => write!(f, "Skip: {reason}"),
        }
    }
}

/// The reason why a message was skipped during processing.
///
/// This enum provides semantic categorization of skip reasons,
/// making it easier to understand pipeline behavior through metrics and logging.
///
/// # Examples
///
/// ```
/// use eidoscope_ingestor::processor::SkipReason;
///
/// let reason = SkipReason::Duplicate;
/// assert_eq!(reason.to_string(), "Duplicate message");
///
/// let reason = SkipReason::Filtered {
///     reason: "Invalid schema".to_string(),
/// };
/// assert_eq!(reason.to_string(), "Filtered: Invalid schema");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum SkipReason {
    /// The message is a duplicate of one already processed.
    Duplicate,

    /// The message was filtered out by business logic.
    Filtered {
        /// Human-readable explanation of why the message was filtered.
        reason: String,
    },

    /// The message format or content is invalid.
    Invalid {
        /// Description of what makes the message invalid.
        reason: String,
    },

    /// The message is outside the acceptable time window.
    OutOfWindow {
        /// Explanation of the time window violation.
        reason: String,
    },

    /// The message failed to meet rate limiting criteria.
    RateLimited,

    /// A custom skip reason not covered by other variants.
    ///
    /// Use this for application-specific skip logic.
    Custom {
        /// Category or type of the custom reason.
        category: String,
        /// Detailed explanation.
        details: String,
    },
}

impl fmt::Display for SkipReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Duplicate => write!(f, "Duplicate message"),
            Self::Filtered { reason } => write!(f, "Filtered: {reason}"),
            Self::Invalid { reason } => write!(f, "Invalid: {reason}"),
            Self::OutOfWindow { reason } => write!(f, "Out of window: {reason}"),
            Self::RateLimited => write!(f, "Rate limited"),
            Self::Custom { category, details } => write!(f, "{category}: {details}"),
        }
    }
}

impl SkipReason {
    /// Creates a new filtered skip reason.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::processor::SkipReason;
    ///
    /// let reason = SkipReason::filtered("Below minimum threshold");
    /// ```
    #[must_use]
    pub fn filtered(reason: impl Into<String>) -> Self {
        Self::Filtered {
            reason: reason.into(),
        }
    }

    /// Creates a new invalid skip reason.
    #[must_use]
    pub fn invalid(reason: impl Into<String>) -> Self {
        Self::Invalid {
            reason: reason.into(),
        }
    }

    /// Creates a new out-of-window skip reason.
    #[must_use]
    pub fn out_of_window(reason: impl Into<String>) -> Self {
        Self::OutOfWindow {
            reason: reason.into(),
        }
    }

    /// Creates a new custom skip reason.
    #[must_use]
    pub fn custom(category: impl Into<String>, details: impl Into<String>) -> Self {
        Self::Custom {
            category: category.into(),
            details: details.into(),
        }
    }
}
