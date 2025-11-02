//! Message metadata types.
//!
//! This module provides validated newtypes for message metadata,
//! implementing type-safe wrappers for timestamps and source identifiers.

use crate::error::{Result, ValidationError};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// A validated timestamp for message creation or reception.
///
/// This newtype ensures timestamps are valid and not in the future,
/// implementing the "constrained types" pattern.
///
/// # Examples
///
/// ```
/// use eidoscope_ingestor::message::Timestamp;
/// use std::time::SystemTime;
///
/// let now = SystemTime::now();
/// let timestamp = Timestamp::now();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(SystemTime);

impl Timestamp {
    /// Creates a new `Timestamp` with the current system time.
    ///
    /// This is the primary constructor for timestamps and cannot fail.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::message::Timestamp;
    ///
    /// let ts = Timestamp::now();
    /// ```
    #[must_use]
    pub fn now() -> Self {
        Self(SystemTime::now())
    }

    /// Creates a `Timestamp` from a `SystemTime`, validating it's not in the future.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidTimestamp`] if the timestamp is in the future.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::message::Timestamp;
    /// use std::time::{SystemTime, Duration};
    ///
    /// let past = SystemTime::now() - Duration::from_secs(60);
    /// let timestamp = Timestamp::from_system_time(past)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_system_time(time: SystemTime) -> Result<Self> {
        // Allow a small grace period (1 second) for clock skew
        let grace_period = std::time::Duration::from_secs(1);
        let now = SystemTime::now();

        if let Ok(duration) = time.duration_since(now) {
            if duration > grace_period {
                return Err(ValidationError::InvalidTimestamp {
                    field: "timestamp".to_string(),
                    reason: "Timestamp cannot be in the future".to_string(),
                }
                .into());
            }
        }

        Ok(Self(time))
    }

    /// Returns the inner `SystemTime` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::message::Timestamp;
    ///
    /// let ts = Timestamp::now();
    /// let system_time = ts.as_system_time();
    /// ```
    #[must_use]
    pub fn as_system_time(&self) -> SystemTime {
        self.0
    }

    /// Converts the timestamp into its inner `SystemTime`.
    #[must_use]
    pub fn into_system_time(self) -> SystemTime {
        self.0
    }
}

impl From<Timestamp> for SystemTime {
    fn from(ts: Timestamp) -> Self {
        ts.0
    }
}

/// A validated source identifier for messages.
///
/// This newtype represents where a message originated from,
/// ensuring source names are non-empty and properly formatted.
///
/// # Examples
///
/// ```
/// use eidoscope_ingestor::message::SourceName;
///
/// let source = SourceName::new("kafka-topic-events")?;
/// assert_eq!(source.as_str(), "kafka-topic-events");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SourceName(String);

impl SourceName {
    /// Maximum allowed length for a source name.
    pub const MAX_LENGTH: usize = 128;

    /// Creates a new `SourceName` from a string.
    ///
    /// # Validation
    ///
    /// - The name must not be empty after trimming whitespace
    /// - The name must not exceed [`MAX_LENGTH`](Self::MAX_LENGTH) characters
    /// - The name should contain only alphanumeric characters, hyphens, and underscores
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError`] if validation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::message::SourceName;
    ///
    /// let source = SourceName::new("my-source")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        let trimmed = name.trim();

        if trimmed.is_empty() {
            return Err(ValidationError::EmptyValue {
                field: "source_name".to_string(),
            }
            .into());
        }

        if trimmed.len() > Self::MAX_LENGTH {
            return Err(ValidationError::TooLong {
                field: "source_name".to_string(),
                max_length: Self::MAX_LENGTH,
                actual_length: trimmed.len(),
            }
            .into());
        }

        // Validate format: alphanumeric, hyphens, underscores, dots, slashes
        if !trimmed.chars().all(|c| {
            c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/'
        }) {
            return Err(ValidationError::InvalidFormat {
                field: "source_name".to_string(),
                reason: "Source name must contain only alphanumeric characters, hyphens, underscores, dots, and slashes".to_string(),
            }
            .into());
        }

        Ok(Self(trimmed.to_string()))
    }

    /// Returns the source name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Converts the `SourceName` into its inner `String`.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<str> for SourceName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SourceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Complete metadata for a message.
///
/// This aggregates all metadata about a message into a single, cohesive type.
///
/// # Examples
///
/// ```
/// use eidoscope_ingestor::message::{MessageMetadata, SourceName, Timestamp};
///
/// let metadata = MessageMetadata {
///     timestamp: Timestamp::now(),
///     source: SourceName::new("my-source")?,
/// };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// When the message was created or received.
    pub timestamp: Timestamp,

    /// Where the message originated from.
    pub source: SourceName,
}

impl MessageMetadata {
    /// Creates new metadata with the current timestamp and given source.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::message::{MessageMetadata, SourceName};
    ///
    /// let source = SourceName::new("kafka-events")?;
    /// let metadata = MessageMetadata::new(source);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn new(source: SourceName) -> Self {
        Self {
            timestamp: Timestamp::now(),
            source,
        }
    }

    /// Creates new metadata with a specific timestamp and source.
    #[must_use]
    pub fn with_timestamp(timestamp: Timestamp, source: SourceName) -> Self {
        Self { timestamp, source }
    }
}
