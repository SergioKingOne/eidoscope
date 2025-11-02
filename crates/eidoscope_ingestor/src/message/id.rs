//! Message identifier types.
//!
//! This module provides the [`MessageId`] newtype, implementing the
//! single-case union pattern for type-safe message identification.

use crate::error::{Result, ValidationError};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A validated, unique message identifier.
///
/// This newtype wraps a string and ensures it is non-empty and contains
/// valid identifier characters. It implements the "validated construction"
/// pattern - invalid identifiers cannot be created.
///
/// # Examples
///
/// ```
/// use eidoscope_ingestor::message::MessageId;
///
/// // Valid construction
/// let id = MessageId::new("msg-12345").unwrap();
/// assert_eq!(id.as_str(), "msg-12345");
///
/// // Invalid construction fails
/// assert!(MessageId::new("").is_err());
/// assert!(MessageId::new("   ").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MessageId(String);

impl MessageId {
    /// Maximum allowed length for a message ID.
    pub const MAX_LENGTH: usize = 256;

    /// Creates a new `MessageId` from a string.
    ///
    /// # Validation
    ///
    /// - The ID must not be empty after trimming whitespace
    /// - The ID must not exceed [`MAX_LENGTH`](Self::MAX_LENGTH) characters
    /// - The ID should contain only printable ASCII characters
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::EmptyValue`] if the ID is empty or whitespace.
    /// Returns [`ValidationError::TooLong`] if the ID exceeds maximum length.
    /// Returns [`ValidationError::InvalidFormat`] if the ID contains invalid characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::message::MessageId;
    ///
    /// let id = MessageId::new("message-001")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(id: impl Into<String>) -> Result<Self> {
        let id = id.into();
        let trimmed = id.trim();

        // Validate non-empty
        if trimmed.is_empty() {
            return Err(ValidationError::EmptyValue {
                field: "message_id".to_string(),
            }
            .into());
        }

        // Validate length
        if trimmed.len() > Self::MAX_LENGTH {
            return Err(ValidationError::TooLong {
                field: "message_id".to_string(),
                max_length: Self::MAX_LENGTH,
                actual_length: trimmed.len(),
            }
            .into());
        }

        // Validate format: only printable ASCII, no control characters
        if !trimmed
            .chars()
            .all(|c| c.is_ascii() && !c.is_ascii_control())
        {
            return Err(ValidationError::InvalidFormat {
                field: "message_id".to_string(),
                reason: "ID must contain only printable ASCII characters".to_string(),
            }
            .into());
        }

        Ok(Self(trimmed.to_string()))
    }

    /// Creates a new `MessageId` from a string, generating a UUID if the input is empty.
    ///
    /// This is a convenience constructor for cases where an ID must always be available.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::message::MessageId;
    ///
    /// let id = MessageId::new_or_generate("");
    /// assert!(!id.as_str().is_empty());
    /// ```
    #[must_use]
    pub fn new_or_generate(id: impl Into<String>) -> Self {
        let id = id.into();
        Self::new(&id).unwrap_or_else(|_| {
            // Generate a simple UUID-like string
            let uuid = format!(
                "msg-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_nanos())
            );
            Self(uuid)
        })
    }

    /// Returns the message ID as a string slice.
    ///
    /// This is the primary way to access the inner value of a `MessageId`.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::message::MessageId;
    ///
    /// let id = MessageId::new("msg-001")?;
    /// assert_eq!(id.as_str(), "msg-001");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Converts the `MessageId` into its inner `String`.
    ///
    /// This consumes the `MessageId` and returns the underlying string value.
    ///
    /// # Examples
    ///
    /// ```
    /// use eidoscope_ingestor::message::MessageId;
    ///
    /// let id = MessageId::new("msg-001")?;
    /// let s: String = id.into_string();
    /// assert_eq!(s, "msg-001");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for MessageId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for MessageId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
