//! SurrealDB persistence adapter and transaction executor abstractions.
use async_trait::async_trait;
use forma_core::domain::entities::{Created, FormaError, Saved};
use std::{borrow::Cow, marker::PhantomData};
use surrealdb::{
    Connection, Surreal, engine,
    method::{Create, Delete, Insert, Query, Select, Transaction, Update, Upsert},
    opt::{CreateResource, IntoResource},
};

use crate::domain::{blueprints::{AccountRepositoryBlueprint, RepositoryBlueprint}, entities::{AccountTable, UserTable}};

/// Provides SurrealDB operations through an abstract executor.
pub struct SurrealdbAdaptor<E, C> {
    /// Executor used for database operations.
    executor: E,
    /// Connection type associated with the executor.
    _conn: PhantomData<C>
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
    E: SurrealExecutor<C>,
{
    /// Creates an account record.
    ///
    /// # Arguments
    /// * `account` - Account data in the created state.
    ///
    /// # Returns
    /// The saved account record.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if record creation fails.
    async fn create_account(
        &self,
        account: AccountTable<Created>,
    ) -> Result<AccountTable<Saved>, FormaError> {

    }

    /// Associates an OAuth provider identity with an account.
    ///
    /// # Arguments
    /// * `account_id` - Account identifier.
    /// * `provider` - OAuth provider.
    /// * `provider_id` - Provider-assigned identity.
    ///
    /// # Returns
    /// `()` when the association is stored.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the update fails.
    async fn add_oauth_provider(
        &self,
        account_id: &str,
        provider: Provider,
        provider_id: &str,
    ) -> Result<(), FormaError> {

    }

    //async fn is_account_exists(&self, account_id: &str) -> Result<bool, FormaError>;

    /// Retrieves an account by its identifier.
    ///
    /// # Arguments
    /// * `account_id` - Account identifier.
    ///
    /// # Returns
    /// The saved account if it exists.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the lookup fails.
    async fn get_account(
        &self,
        account_id: &str,
    ) -> Result<Option<AccountTable<Saved>>, FormaError> {}

    /// Retrieves an account by its provider identity.
    ///
    /// # Arguments
    /// * `provider` - OAuth provider.
    /// * `provider_id` - Provider-assigned identity.
    ///
    /// # Returns
    /// The matching saved account, if any.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the lookup fails.
    async fn get_account_by_oauth_provider(
        &self,
        provider: Provider,
        provider_id: &str,
    ) -> Result<Option<AccountTable<Saved>>, FormaError> {}

    /// Updates account fields.
    ///
    /// # Arguments
    /// * `account_id` - Account identifier.
    /// * `new_data` - Replacement account fields.
    ///
    /// # Returns
    /// `()` when the update succeeds.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the update fails.
    async fn update_account(
        &self,
        account_id: &str,
        new_data: HashMap<String, String>,
    ) -> Result<(), FormaError> {}

    /// Creates and associates a user with an account.
    ///
    /// # Arguments
    /// * `account_id` - Owning account identifier.
    /// * `user` - User data in the created state.
    ///
    /// # Returns
    /// The saved user record.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if user creation or association fails.
    async fn init_user(
        &self,
        account_id: &str,
        user: UserTable<Created>,
    ) -> Result<UserTable<Saved>, FormaError> {}

    /// Retrieves a user by identifier.
    ///
    /// # Arguments
    /// * `user_id` - User identifier.
    ///
    /// # Returns
    /// The saved user record.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the lookup fails.
    async fn get_user(&self, user_id: &str) -> Result<UserTable<Saved>, FormaError> {}

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
    /// Returns a [`FormaError`] if the update fails.
    async fn hard_delete_oauth_provider(
        &self,
        account_id: &str,
        provider: Provider,
    ) -> Result<(), FormaError> {}

    //async fn delete_account(&self, account_id: &str) -> Result<(), FormaError>;
}

/// Owns a SurrealDB client and implements the common executor interface.
pub struct SurrealDatabase<C: Connection>(Surreal<C>);

/// Owns a SurrealDB transaction and implements the common executor interface.
pub struct SurrealTransaction<C: Connection>(Transaction<C>);

impl<C: Connection> SurrealDatabase<C> {
    /// Wraps a SurrealDB client as a database executor.
    ///
    /// # Arguments
    /// * `db` - Connected SurrealDB client.
    ///
    /// # Returns
    /// A database wrapper.
    pub fn new(db: Surreal<C>) -> Self {
        Self(db)
    }

    /// Begins a transaction on the underlying database.
    ///
    /// # Returns
    /// A transaction wrapper.
    ///
    /// # Errors
    /// Returns a SurrealDB error if the transaction cannot be started.
    pub async fn begin(&self) -> surrealdb::Result<SurrealTransaction<C>> {
        let tx = self.0.clone().begin().await?;
        Ok(SurrealTransaction(tx))
    }
}

impl<C: Connection> SurrealTransaction<C> {
    /// Commits this transaction.
    ///
    /// # Returns
    /// `()` when the transaction is committed.
    ///
    /// # Errors
    /// Returns a SurrealDB error if the commit fails.
    pub async fn commit(self) -> surrealdb::Result<()> {
        self.0.commit().await?;
        Ok(())
    }

    /// Cancels this transaction.
    ///
    /// # Returns
    /// `()` when the transaction is cancelled.
    ///
    /// # Errors
    /// Returns a SurrealDB error if cancellation fails.
    pub async fn cancel(self) -> surrealdb::Result<()> {
        self.0.cancel().await?;
        Ok(())
    }
}

/// Defines the SurrealDB operations supported by database and transaction wrappers.
pub trait SurrealExecutor<C: Connection> {
    /// Creates a query builder for the supplied query text.
    ///
    /// # Arguments
    /// * `query` - Query text or owned query string.
    ///
    /// # Returns
    /// A SurrealDB query builder borrowing this executor.
    fn query<'client>(&'client self, query: impl Into<Cow<'client, str>>) -> Query<'client, C>;

    /// Creates a select builder for a resource.
    ///
    /// # Arguments
    /// * `resource` - Resource to select.
    ///
    /// # Returns
    /// A select builder.
    fn select<O>(&self, resource: impl IntoResource<O>) -> Select<C, O>;

    /// Creates a record-creation builder for a resource.
    ///
    /// # Arguments
    /// * `resource` - Resource to create.
    ///
    /// # Returns
    /// A create builder.
    fn create<R>(&self, resource: impl CreateResource<R>) -> Create<C, R>;

    /// Creates an insert builder for a resource.
    ///
    /// # Arguments
    /// * `resource` - Resource to insert.
    ///
    /// # Returns
    /// An insert builder.
    fn insert<O>(&self, resource: impl IntoResource<O>) -> Insert<C, O>;

    /// Creates an upsert builder for a resource.
    ///
    /// # Arguments
    /// * `resource` - Resource to upsert.
    ///
    /// # Returns
    /// An upsert builder.
    fn upsert<O>(&self, resource: impl IntoResource<O>) -> Upsert<C, O>;

    /// Creates an update builder for a resource.
    ///
    /// # Arguments
    /// * `resource` - Resource to update.
    ///
    /// # Returns
    /// An update builder.
    fn update<O>(&self, resource: impl IntoResource<O>) -> Update<C, O>;

    /// Creates a delete builder for a resource.
    ///
    /// # Arguments
    /// * `resource` - Resource to delete.
    ///
    /// # Returns
    /// A delete builder.
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
