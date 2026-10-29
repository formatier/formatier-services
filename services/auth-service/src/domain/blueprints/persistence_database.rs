//! Persistence contracts for accounts, users, and authentication sessions.
use std::collections::HashMap;

use crate::domain::entities::{AccountTable, Provider, ProviderIdent, SessionMetadata, SessionTable, UserTable};
use async_trait::async_trait;
use forma_core::domain::entities::{Created, FormaError, Saved};

/// Combines the account and token repository contracts.
pub trait RepositoryBlueprint: AccountRepositoryBlueprint + TokenRepositoryBlueprint {}

#[async_trait]
/// Defines persistence operations for accounts and users.
pub trait AccountRepositoryBlueprint {
    /// Persists a newly created account.
    async fn create_account(
        &self,
        account: AccountTable<Created>,
    ) -> Result<AccountTable<Saved>, FormaError>;

    /// Associates an OAuth identity with an account.
    async fn add_oauth_provider(
        &self,
        account_id: &str,
        provider: ProviderIdent,
        provider_id: &str,
        provider_email: &str
    ) -> Result<(), FormaError>;

    //async fn is_account_exists(&self, account_id: &str) -> Result<bool, FormaError>;

    /// Retrieves an account by its identifier.
    async fn get_account(
        &self,
        account_id: &str,
    ) -> Result<Option<AccountTable<Saved>>, FormaError>;

    /// Retrieves an account by an associated OAuth identity.
    async fn get_account_by_oauth_provider(
        &self,
        provider_id: &str,
    ) -> Result<Option<AccountTable<Saved>>, FormaError>;

    /// Updates account fields from string-valued data.
    async fn update_account(
        &self,
        account_id: &str,
        new_data: HashMap<String, String>,
    ) -> Result<(), FormaError>;

    /// Creates a user associated with an account.
    async fn init_user(
        &self,
        account_id: &str,
        user: UserTable<Created>,
    ) -> Result<UserTable<Saved>, FormaError>;

    /// Retrieves a persisted user by identifier.
    async fn get_user(&self, user_id: &str) -> Result<UserTable<Saved>, FormaError>;

    //async fn get_user_by_account_id(&self, account_id: &str) -> Result<UserTable, FormaError>;

    //async fn update_user(&self, account_id: &str, user: &UserTable) -> Result<(), FormaError>;

    /// Removes an OAuth provider association from an account.
    async fn hard_delete_oauth_provider(
        &self,
        account_id: &str,
        provider: ProviderIdent,
    ) -> Result<(), FormaError>;

    //async fn delete_account(&self, account_id: &str) -> Result<(), FormaError>;
}

#[async_trait]
/// Defines persistence operations for authentication sessions.
pub trait TokenRepositoryBlueprint {
    /// Persists a newly created session.
    async fn create_session(
        &self,
        session: SessionTable<Created>,
    ) -> Result<SessionTable<Saved>, FormaError>;

    /// Checks whether a session exists in persistence.
    async fn is_session_available(&self, session_id: &str) -> Result<bool, FormaError>;

    /// Retrieves a session by identifier.
    async fn get_session(&self, session_id: &str) -> Result<SessionTable<Saved>, FormaError>;

    /// Atomically advances the refresh-token generation.
    async fn rotate_refresh_token(
        &self,
        session_id: &str,
        expected_version: u64,
    ) -> Result<bool, FormaError>;

    /// Retrieves all saved sessions owned by an account.
    async fn get_sessions(
        &self,
        account_id: &str,
    ) -> Result<Box<dyn Iterator<Item = SessionTable<Saved>> + Send>, FormaError>;

    /// Deletes a session from persistence.
    async fn delete_session(&self, session_id: &str) -> Result<(), FormaError>;
}
