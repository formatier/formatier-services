//! Mail content and delivery data types.
use forma_core::inline_mod;

inline_mod!(dto);

/// Represents a message submitted for delivery through an external mail service.
pub struct ExternalMail {
    /// Message subject.
    pub subject: Box<str>,
    /// Message body.
    pub body: Box<str>,
    /// Recipient address.
    pub destination: Box<str>,
    /// End-to-end delivery options.
    pub extensions: ExternalMailE2eExtension,
}

/// Optional extensions supported for an external mail message.
pub struct ExternalMailExtensions {
    /// Time-to-live in seconds, when specified.
    pub ttl: Option<u32>,
    /// Optional end-to-end encryption settings.
    pub e2e: Option<ExternalMailE2eExtension>,
}

/// End-to-end encryption options for external mail delivery.
pub struct ExternalMailE2eExtension {}
