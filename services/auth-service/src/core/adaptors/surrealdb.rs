//! SurrealDB persistence adapter and transaction executor abstractions.
use async_trait::async_trait;
use auth_service::domain::entities::Session;
use forma_core::domain::entities::{
    Created, CreatedTable, FormaError, FormaErrorApp, FormaErrorConverter, FormaErrorDatabase,
    FormaErrorFlattener, FormaErrorKind, FormaErrorSurrealExt, FormaOptionConverter, Saved,
    UpdaterFilter,
};
use serde::Serialize;
use std::{borrow::Cow, collections::HashMap, marker::PhantomData, str::FromStr};
use surrealdb::{
    Connection, Surreal, engine,
    method::{Create, Delete, Insert, Query, Select, Transaction, Update, Upsert},
    opt::{CreateResource, IntoResource},
    types::{self, RecordId, SurrealValue, ToSql, vars},
};

use crate::domain::{
    blueprints::{AccountRepositoryBlueprint, RepositoryBlueprint, TokenRepositoryBlueprint},
    entities::{
        AccountTable, AccountUpdaterFilter, Provider, ProviderIdent, SessionTable, UserTable,
    },
};

macro_rules! use_query {
    ($name:literal) => {
        include_str!(concat!("./queries/", $name, ".surql"))
    };
}

#[derive(SurrealValue)]
struct Retrieves {
    id: RecordId,
}

fn to_account_id(account_id: &str) -> Result<types::Uuid, FormaError> {
    types::Uuid::from_str(account_id)
        .map_forma_err(FormaErrorDatabase::InvalidObjectId, "failed to parse ID")
}

fn to_user_id(account_id: &str) -> Result<types::Uuid, FormaError> {
    types::Uuid::from_str(account_id)
        .map_forma_err(FormaErrorDatabase::InvalidObjectId, "failed to parse ID")
}

fn to_session_id(account_id: &str) -> Result<types::Uuid, FormaError> {
    types::Uuid::from_str(account_id)
        .map_forma_err(FormaErrorDatabase::InvalidObjectId, "failed to parse ID")
}

/// Provides SurrealDB operations through an abstract executor.
pub struct SurrealdbAdaptor<E, C> {
    /// Executor used for database operations.
    executor: E,
    /// Connection type associated with the executor.
    _conn: PhantomData<C>,
}

impl<E, C> SurrealdbAdaptor<E, C>
where
    C: Connection,
    E: SurrealExecutor<C>,
{
}

#[async_trait]
impl<E, C> AccountRepositoryBlueprint for SurrealdbAdaptor<E, C>
where
    C: Connection,
    E: SurrealExecutor<C> + Send + Sync,
{
    /// Creates an account record.
    async fn create_account(
        &self,
        account: AccountTable<Created>,
    ) -> Result<AccountTable<Saved>, FormaError> {
        let account_id = types::Uuid::new_v4();
        let ret: Retrieves = self
            .executor
            .create(RecordId::new("account", account_id))
            .content(account.clone())
            .await
            .map_surreal_forma_err(
                FormaErrorDatabase::InsertError,
                "failed to insert an account into the database",
            )
            .flatten_forma()?;

        Ok(account.with_id(ret.id))
    }

    /// Associates an OAuth provider identity with an account.
    async fn add_oauth_provider(
        &self,
        account_id: &str,
        provider: ProviderIdent,
        provider_id: &str,
        provider_email: &str,
    ) -> Result<(), FormaError> {
        let account_id = to_account_id(account_id)?;
        let _ret = self
            .executor
            .query(use_query!("add_oauth_provider"))
            .bind(vars! {
                account_key: RecordId::new("account", account_id),
                provider: provider,
                provider_key: RecordId::new("provider", provider_id),
                email: provider_email
            })
            .await
            .map(|v| v.check())
            .flatten()
            .map_surreal_forma_err(
                FormaErrorDatabase::UpdateError,
                "failed to add the OAuth provider to the account",
            )?;

        Ok(())
    }

    //async fn is_account_exists(&self, account_id: &str) -> Result<bool, FormaError>;

    /// Retrieves an account by its identifier.
    async fn get_account(
        &self,
        account_id: &str,
    ) -> Result<Option<AccountTable<Saved>>, FormaError> {
        let account_id = to_account_id(account_id)?;
        let ret = self
            .executor
            .select(RecordId::new("account", account_id))
            .await
            .map_surreal_forma_err(
                FormaErrorDatabase::SelectError,
                "failed to select an account",
            )?;

        Ok(ret)
    }

    /// Retrieves an account by its provider identity.
    async fn get_account_by_oauth_provider(
        &self,
        provider_id: &str,
    ) -> Result<Option<AccountTable<Saved>>, FormaError> {
        let ret = self
            .executor
            .query(use_query!("get_account_by_oauth_provider"))
            .bind(vars! {
                provider_key: RecordId::new("provider", provider_id)
            })
            .await
            .map(|v| v.check())
            .flatten()
            .map(|mut v| v.take(0))
            .flatten()
            .map_surreal_forma_err(
                FormaErrorDatabase::SelectError,
                "failed to find an account with the OAuth provider ID",
            )?;

        Ok(ret)
    }

    /// Updates account fields.
    async fn update_account(
        &self,
        account_id: &str,
        new_data: HashMap<String, String>,
    ) -> Result<(), FormaError> {
        let account_id = to_account_id(account_id)?;
        let new_data: HashMap<_, _> =
            AccountUpdaterFilter::filter_iter(new_data.into_iter()).collect();

        let _ret = self
            .executor
            .query(use_query!("update_account"))
            .bind(vars! {
                account_key: RecordId::new("account", account_id),
                data_to_patch: new_data,
            })
            .await
            .map(|v| v.check())
            .flatten()
            .map_surreal_forma_err(
                FormaErrorDatabase::UpdateError,
                "failed to update the account with the provided data",
            )?;

        Ok(())
    }

    /// Creates and associates a user with an account.
    async fn init_user(
        &self,
        account_id: &str,
        user: UserTable<Created>,
    ) -> Result<UserTable<Saved>, FormaError> {
        let account_id = to_account_id(account_id)?;
        let user_id = types::Uuid::new_v7();
        let user_key = RecordId::new("user", user_id);

        let _ret = self
            .executor
            .query(use_query!("init_user"))
            .bind(vars! {
                user_key: user_key.clone(),
                account_key: RecordId::new("account", account_id),
                data: user.clone(),
            })
            .await
            .map_surreal_forma_err(
                FormaErrorDatabase::InsertError,
                "failed to insert a user into the database",
            )?;

        Ok(user.with_id(user_key))
    }

    /// Retrieves a user by identifier.
    async fn get_user(&self, user_id: &str) -> Result<UserTable<Saved>, FormaError> {
        let user_id = to_user_id(user_id)?;
        let ret = self
            .executor
            .select(RecordId::new("user", user_id))
            .await
            .map_surreal_forma_err(FormaErrorDatabase::SelectError, "failed to select a user")?
            .ok_or_forma_error()?;

        Ok(ret)
    }

    //async fn get_user_by_account_id(&self, account_id: &str) -> Result<UserTable, FormaError>;

    //async fn update_user(&self, account_id: &str, user: &UserTable) -> Result<(), FormaError>;

    /// Removes an OAuth provider association from an account.
    async fn hard_delete_oauth_provider(
        &self,
        account_id: &str,
        provider: Provider,
    ) -> Result<(), FormaError> {
        let account_id = to_account_id(account_id)?;
        let _ret = self
            .executor
            .query(use_query!("hard_delete_oauth_provider"))
            .bind(vars! {
                account_key: RecordId::new("account", account_id),
                provider: provider
            })
            .await
            .map(|v| v.check())
            .flatten()
            .map_surreal_forma_err(
                FormaErrorDatabase::DeleteError,
                "failed to delete an oauth provider",
            )?;

        Ok(())
    }

    //async fn delete_account(&self, account_id: &str) -> Result<(), FormaError>;
}

#[async_trait]
impl<E, C> TokenRepositoryBlueprint for SurrealdbAdaptor<E, C>
where
    C: Connection,
    E: SurrealExecutor<C> + Send + Sync,
{
    /// Persists a newly created session.
    async fn create_session(
        &self,
        session: SessionTable<Created>,
    ) -> Result<SessionTable<Saved>, FormaError> {
        let session_id = types::Uuid::new_v4();
        let ret: Retrieves = self
            .executor
            .create(RecordId::new("session", session_id))
            .content(session.clone())
            .await
            .map_surreal_forma_err(
                FormaErrorDatabase::InsertError,
                "failed to insert an session into the database",
            )
            .flatten_forma()?;

        Ok(session.with_id(ret.id))
    }

    /// Checks whether a session exists in persistence.
    async fn is_session_available(&self, session_id: &str) -> Result<bool, FormaError> {
        let session_id = to_session_id(session_id)?;
        let ret: Option<bool> = self
            .executor
            .query(use_query!("is_session_available"))
            .bind(vars! {
                session_key: RecordId::new("session", session_id)
            })
            .await
            .map(|mut v| v.take(0))
            .flatten()
            .map_forma_err(FormaErrorDatabase::SelectError, "failed to search session")?;

        Ok(ret.unwrap_or_default())
    }

    /// Updates a session's refresh timestamp.
    async fn refresh_session(&self, session_id: &str) -> Result<(), FormaError> {
        let session_id = to_session_id(session_id)?;
        let _ret = self
            .executor
            .query(use_query!("refresh_session"))
            .bind(vars! {
                session_key: RecordId::new("session", session_id)
            })
            .await
            .map(|v| v.check())
            .flatten()
            .map_forma_err(FormaErrorDatabase::SelectError, "failed to search session")?;

        Ok(())
    }

    /// Retrieves all saved sessions owned by an account.
    async fn get_sessions(
        &self,
        account_id: &str,
    ) -> Result<Box<dyn Iterator<Item = SessionTable<Saved>> + Send>, FormaError> {
        let account_id = to_account_id(account_id)?;
        let ret: Vec<SessionTable<Saved>> = self
            .executor
            .query(use_query!("refresh_session"))
            .bind(vars! {
                account_key: RecordId::new("account", account_id)
            })
            .await
            .map(|mut v| v.take(0))
            .flatten()
            .map_forma_err(FormaErrorDatabase::SelectError, "failed to search session")?;

        Ok(Box::new(ret.into_iter()))
    }

    /// Deletes a session from persistence.
    async fn delete_session(&self, session_id: &str) -> Result<(), FormaError> {
        let session_id = to_session_id(session_id)?;
        let _ret = self
            .executor
            .query(use_query!("delete_session"))
            .bind(vars! {
                session_key: RecordId::new("session", session_id)
            })
            .await
            .map(|v| v.check())
            .flatten()
            .map_surreal_forma_err(FormaErrorDatabase::SelectError, "failed to select a user")?;

        Ok(())
    }
}

/// Owns a SurrealDB client and implements the common executor interface.
pub struct SurrealDatabase<C: Connection>(Surreal<C>);

/// Owns a SurrealDB transaction and implements the common executor interface.
pub struct SurrealTransaction<C: Connection>(Transaction<C>);

impl<C: Connection> SurrealDatabase<C> {
    /// Wraps a SurrealDB client as a database executor.
    pub fn new(db: Surreal<C>) -> Self {
        Self(db)
    }

    /// Begins a transaction on the underlying database.
    pub async fn begin(&self) -> surrealdb::Result<SurrealTransaction<C>> {
        let tx = self.0.clone().begin().await?;
        Ok(SurrealTransaction(tx))
    }
}

impl<C: Connection> SurrealTransaction<C> {
    /// Commits this transaction.
    pub async fn commit(self) -> surrealdb::Result<()> {
        self.0.commit().await?;
        Ok(())
    }

    /// Cancels this transaction.
    pub async fn cancel(self) -> surrealdb::Result<()> {
        self.0.cancel().await?;
        Ok(())
    }
}

/// Defines the SurrealDB operations supported by database and transaction wrappers.
pub trait SurrealExecutor<C: Connection> {
    /// Creates a query builder for the supplied query text.
    fn query<'client>(&'client self, query: impl Into<Cow<'client, str>>) -> Query<'client, C>;

    /// Creates a select builder for a resource.
    fn select<O>(&self, resource: impl IntoResource<O>) -> Select<C, O>;

    /// Creates a record-creation builder for a resource.
    fn create<R>(&self, resource: impl CreateResource<R>) -> Create<C, R>;

    /// Creates an insert builder for a resource.
    fn insert<O>(&self, resource: impl IntoResource<O>) -> Insert<C, O>;

    /// Creates an upsert builder for a resource.
    fn upsert<O>(&self, resource: impl IntoResource<O>) -> Upsert<C, O>;

    /// Creates an update builder for a resource.
    fn update<O>(&self, resource: impl IntoResource<O>) -> Update<C, O>;

    /// Creates a delete builder for a resource.
    fn delete<O>(&self, resource: impl IntoResource<O>) -> Delete<C, O>;
}

impl<C: Connection> SurrealExecutor<C> for SurrealDatabase<C> {
    /// Creates a query builder for the supplied query text.
    fn query<'client>(&'client self, query: impl Into<Cow<'client, str>>) -> Query<'client, C> {
        self.0.query(query)
    }

    /// Creates a select builder for a resource.
    fn select<O>(&self, resource: impl IntoResource<O>) -> Select<C, O> {
        self.0.select(resource)
    }

    /// Creates a record-creation builder for a resource.
    fn create<R>(&self, resource: impl CreateResource<R>) -> Create<C, R> {
        self.0.create(resource)
    }

    /// Creates an insert builder for a resource.
    fn insert<O>(&self, resource: impl IntoResource<O>) -> Insert<C, O> {
        self.0.insert(resource)
    }

    /// Creates an upsert builder for a resource.
    fn upsert<O>(&self, resource: impl IntoResource<O>) -> Upsert<C, O> {
        self.0.upsert(resource)
    }

    /// Creates an update builder for a resource.
    fn update<O>(&self, resource: impl IntoResource<O>) -> Update<C, O> {
        self.0.update(resource)
    }

    /// Creates a delete builder for a resource.
    fn delete<O>(&self, resource: impl IntoResource<O>) -> Delete<C, O> {
        self.0.delete(resource)
    }
}

impl<C: Connection> SurrealExecutor<C> for SurrealTransaction<C> {
    /// Creates a query builder for the supplied query text.
    fn query<'client>(&'client self, query: impl Into<Cow<'client, str>>) -> Query<'client, C> {
        self.0.query(query)
    }

    /// Creates a select builder for a resource.
    fn select<O>(&self, resource: impl IntoResource<O>) -> Select<C, O> {
        self.0.select(resource)
    }

    /// Creates a record-creation builder for a resource.
    fn create<R>(&self, resource: impl CreateResource<R>) -> Create<C, R> {
        self.0.create(resource)
    }

    /// Creates an insert builder for a resource.
    fn insert<O>(&self, resource: impl IntoResource<O>) -> Insert<C, O> {
        self.0.insert(resource)
    }

    /// Creates an upsert builder for a resource.
    fn upsert<O>(&self, resource: impl IntoResource<O>) -> Upsert<C, O> {
        self.0.upsert(resource)
    }

    /// Creates an update builder for a resource.
    fn update<O>(&self, resource: impl IntoResource<O>) -> Update<C, O> {
        self.0.update(resource)
    }

    /// Creates a delete builder for a resource.
    fn delete<O>(&self, resource: impl IntoResource<O>) -> Delete<C, O> {
        self.0.delete(resource)
    }
}
