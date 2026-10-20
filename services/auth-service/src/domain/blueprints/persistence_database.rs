use std::collections::HashMap;

use crate::domain::entities::{AccountModel, Provider, SessionMetadata, SessionModel, UserModel};
use async_trait::async_trait;
use forma_core::domain::entities::{Created, FormaError, Saved};

pub trait RepositoryBlueprint: AccountRepositoryBlueprint + TokenRepositoryBlueprint {}

#[async_trait]
pub trait AccountRepositoryBlueprint {
    async fn create_account(
        &self,
        account: AccountModel<Created>,
    ) -> Result<AccountModel<Saved>, FormaError>;

    async fn add_oauth_provider(
        &self,
        account_id: &str,
        provider: Provider,
        provider_id: &str,
    ) -> Result<(), FormaError>;

    //async fn is_account_exists(&self, account_id: &str) -> Result<bool, FormaError>;

    async fn get_account(
        &self,
        account_id: &str,
    ) -> Result<Option<AccountModel<Saved>>, FormaError>;

    async fn get_account_by_oauth_provider(
        &self,
        provider: Provider,
        provider_id: &str,
    ) -> Result<Option<AccountModel<Saved>>, FormaError>;

    async fn update_account(
        &self,
        account_id: &str,
        new_data: HashMap<String, String>,
    ) -> Result<(), FormaError>;

    async fn init_user(
        &self,
        account_id: &str,
        user: UserModel<Created>,
    ) -> Result<UserModel<Saved>, FormaError>;

    async fn get_user(&self, user_id: &str) -> Result<UserModel<Saved>, FormaError>;

    //async fn get_user_by_account_id(&self, account_id: &str) -> Result<UserModel, FormaError>;

    //async fn update_user(&self, account_id: &str, user: &UserModel) -> Result<(), FormaError>;

    async fn hard_delete_oauth_provider(
        &self,
        account_id: &str,
        provider: Provider,
    ) -> Result<(), FormaError>;

    //async fn delete_account(&self, account_id: &str) -> Result<(), FormaError>;
}

#[async_trait]
pub trait TokenRepositoryBlueprint {
    async fn create_session_metadata(
        &self,
        account_id: &str,
    ) -> Result<SessionMetadata, FormaError>;

    async fn create_session(
        &self,
        session: SessionModel<Created>,
    ) -> Result<SessionModel<Saved>, FormaError>;

    async fn is_session_available(&self, session_id: &str) -> Result<bool, FormaError>;

    async fn refresh_session(&self, session_id: &str) -> Result<(), FormaError>;

    async fn get_sessions(
        &self,
        account_id: &str,
    ) -> Result<Box<dyn Iterator<Item = SessionModel<Saved>> + Send>, FormaError>;

    async fn delete_session(&self, session_id: &str) -> Result<(), FormaError>;
}
