//! Abstraction for cached authentication-token state.
use async_trait::async_trait;
use forma_core::domain::entities::FormaError;

#[async_trait]
/// Defines token-state operations required by authentication services.
pub trait TokenCacheBlueprint {
    /// Creates an active cache entry for a session.
    ///
    /// # Arguments
    /// * `session_id` - Identifier of the session.
    ///
    /// # Returns
    /// `()` when the entry is created.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the cache operation fails.
    async fn create_token_data(&self, session_id: &str) -> Result<(), FormaError>;

    /// Checks whether a session has active cached token state.
    ///
    /// # Arguments
    /// * `session_id` - Identifier of the session.
    ///
    /// # Returns
    /// `true` if the token state is active; otherwise, `false`.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the cache operation fails.
    async fn is_token_available(&self, session_id: &str) -> Result<bool, FormaError>;

    /// Removes or deactivates cached token state for a session.
    ///
    /// # Arguments
    /// * `session_id` - Identifier of the session.
    ///
    /// # Returns
    /// `()` when the cache entry is updated.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the cache operation fails.
    async fn remove_token(&self, session_id: &str) -> Result<(), FormaError>;
}
