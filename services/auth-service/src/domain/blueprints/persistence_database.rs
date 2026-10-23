//! Persistence contracts for accounts, users, and authentication sessions.
use std::collections::HashMap;

use crate::domain::entities::{AccountTable, Provider, SessionMetadata, SessionTable, UserTable};
use async_trait::async_trait;
use forma_core::domain::entities::{Created, FormaError, Saved};

/// Combines the account and token repository contracts.
pub trait RepositoryBlueprint: AccountRepositoryBlueprint + TokenRepositoryBlueprint {}

#[async_trait]
/// Defines persistence operations for accounts and users.
pub trait AccountRepositoryBlueprint {
    /// Persists a newly created account.
    ///
    /// # Arguments
    /// * `account` - Account in the created state.
    ///
    /// # Returns
    /// The persisted account in the saved state.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if persistence fails.
    async fn create_account(
        &self,
        account: AccountTable<Created>,
    ) -> Result<AccountTable<Saved>, FormaError>;

    /// Associates an OAuth identity with an account.
    ///
    /// # Arguments
    /// * `account_id` - Account identifier.
    /// * `provider` - OAuth provider.
    /// * `provider_id` - Provider-assigned user identifier.
    ///
    /// # Returns
    /// `()` when the association is stored.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the account is invalid, unavailable, or
    /// cannot be updated.
    async fn add_oauth_provider(
        &self,
        account_id: &str,
        provider: Provider,
        provider_id: &str,
    ) -> Result<(), FormaError>;

    //async fn is_account_exists(&self, account_id: &str) -> Result<bool, FormaError>;

    /// Retrieves an account by its identifier.
    ///
    /// # Arguments
    /// * `account_id` - Account identifier.
    ///
    /// # Returns
    /// The saved account if it exists; otherwise, `None`.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if lookup or decoding fails.
    async fn get_account(
        &self,
        account_id: &str,
    ) -> Result<Option<AccountTable<Saved>>, FormaError>;

    /// Retrieves an account by an associated OAuth identity.
    ///
    /// # Arguments
    /// * `provider` - OAuth provider.
    /// * `provider_id` - Provider-assigned user identifier.
    ///
    /// # Returns
    /// The saved account if one matches; otherwise, `None`.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if lookup or decoding fails.
    async fn get_account_by_oauth_provider(
        &self,
        provider: Provider,
        provider_id: &str,
    ) -> Result<Option<AccountTable<Saved>>, FormaError>;

    /// Updates account fields from string-valued data.
    ///
    /// # Arguments
    /// * `account_id` - Account identifier.
    /// * `new_data` - Field names and replacement string values.
    ///
    /// # Returns
    /// `()` when the account is updated.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if lookup or update fails.
    async fn update_account(
        &self,
        account_id: &str,
        new_data: HashMap<String, String>,
    ) -> Result<(), FormaError>;

    /// Creates a user associated with an account.
    ///
    /// # Arguments
    /// * `account_id` - Owning account identifier.
    /// * `user` - User data in the created state.
    ///
    /// # Returns
    /// The persisted user in the saved state.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if persistence or account association fails.
    async fn init_user(
        &self,
        account_id: &str,
        user: UserTable<Created>,
    ) -> Result<UserTable<Saved>, FormaError>;

    /// Retrieves a persisted user by identifier.
    ///
    /// # Arguments
    /// * `user_id` - User identifier.
    ///
    /// # Returns
    /// The saved user.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if lookup fails or the user does not exist.
    async fn get_user(&self, user_id: &str) -> Result<UserTable<Saved>, FormaError>;

    //async fn get_user_by_account_id(&self, account_id: &str) -> Result<UserTable, FormaError>;

    //async fn update_user(&self, account_id: &str, user: &UserTable) -> Result<(), FormaError>;

    /// Removes an OAuth provider association from an account.
    ///
    /// # Arguments
    /// * `account_id` - Account identifier.
    /// * `provider` - OAuth provider to remove.
    ///
    /// # Returns
    /// `()` when the association is removed.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the account is unavailable or cannot be
    /// updated.
    async fn hard_delete_oauth_provider(
        &self,
        account_id: &str,
        provider: Provider,
    ) -> Result<(), FormaError>;

    //async fn delete_account(&self, account_id: &str) -> Result<(), FormaError>;
}

#[async_trait]
/// Defines persistence operations for authentication sessions.
pub trait TokenRepositoryBlueprint {
    /// Builds default session metadata for an account.
    ///
    /// # Arguments
    /// * `account_id` - Account that owns the session.
    ///
    /// # Returns
    /// Metadata suitable for creating a session.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the account cannot be found or metadata
    /// cannot be created.
    async fn create_session_metadata(
        &self,
        account_id: &str,
    ) -> Result<SessionMetadata, FormaError>;

    /// Persists a newly created session.
    ///
    /// # Arguments
    /// * `session` - Session in the created state.
    ///
    /// # Returns
    /// The persisted session in the saved state.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if persistence fails.
    async fn create_session(
        &self,
        session: SessionTable<Created>,
    ) -> Result<SessionTable<Saved>, FormaError>;

    /// Checks whether a session exists in persistence.
    ///
    /// # Arguments
    /// * `session_id` - Session identifier.
    ///
    /// # Returns
    /// `true` if the session exists; otherwise, `false`.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the lookup fails.
    async fn is_session_available(&self, session_id: &str) -> Result<bool, FormaError>;

    /// Updates a session's refresh timestamp.
    ///
    /// # Arguments
    /// * `session_id` - Session identifier.
    ///
    /// # Returns
    /// `()` when the session is refreshed.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the session cannot be updated.
    async fn refresh_session(&self, session_id: &str) -> Result<(), FormaError>;

    /// Retrieves all saved sessions owned by an account.
    ///
    /// # Arguments
    /// * `account_id` - Account identifier.
    ///
    /// # Returns
    /// An iterator over the matching saved sessions.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if lookup or decoding fails.
    async fn get_sessions(
        &self,
        account_id: &str,
    ) -> Result<Box<dyn Iterator<Item = SessionTable<Saved>> + Send>, FormaError>;

    /// Deletes a session from persistence.
    ///
    /// # Arguments
    /// * `session_id` - Session identifier.
    ///
    /// # Returns
    /// `()` when the session is deleted.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the session cannot be deleted.
    async fn delete_session(&self, session_id: &str) -> Result<(), FormaError>;
}
