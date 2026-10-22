use std::borrow::Cow;
use async_trait::async_trait;
use auth_service::domain::blueprints::AccountRepositoryBlueprint;
use surrealdb::{
    Connection, Surreal, method::{Create, Delete, Insert, Query, Select, Transaction, Update, Upsert}, opt::{CreateResource, IntoResource},
};

use crate::domain::blueprints::RepositoryBlueprint;

pub struct SurrealDatabase<C: Connection> {
    db: Surreal<C>,
}

pub struct SurrealTransaction<C: Connection> {
    tx: Transaction<C>,
}

pub trait SurrealExecutor<C: Connection> {
    fn query<'client>(&'client self, query: impl Into<Cow<'client, str>>) -> Query<'client, C>;
    fn select<O>(&self, resource: impl IntoResource<O>) -> Select<C, O>;
    fn create<R>(&self, resource: impl CreateResource<R>) -> Create<C, R>;
    fn insert<O>(&self, resource: impl IntoResource<O>) -> Insert<C, O>;
    fn upsert<O>(&self, resource: impl IntoResource<O>) -> Upsert<C, O>;
    fn update<O>(&self, resource: impl IntoResource<O>) -> Update<'_, C, O>;
    fn delete<O>(&self, resource: impl IntoResource<O>) -> Delete<'_, C, O>;
}

impl<C: Connection> SurrealExecutor<C> for SurrealDatabase<C> {
    fn query<'client>(&'client self, query: impl Into<Cow<'client, str>>) -> Query<'client, C> {
        self.db.query(query)
    }

    fn select<O>(&self, resource: impl IntoResource<O>) -> Select<C, O> {
        self.db.select(resource)
    }

    fn create<R>(&self, resource: impl CreateResource<R>) -> Create<C, R> {
        self.db.create(resource)
    }

    fn insert<O>(&self, resource: impl IntoResource<O>) -> Insert<C, O> {
        self.db.insert(resource)
    }

    fn upsert<O>(&self, resource: impl IntoResource<O>) -> Upsert<C, O> {
        self.db.upsert(resource)
    }

    fn update<O>(&self, resource: impl IntoResource<O>) -> Update<C, O> {
        self.db.update(resource)
    }

    fn delete<O>(&self, resource: impl IntoResource<O>) -> Delete<C, O> {
        self.db.delete(resource)
    }
}

impl<C: Connection> SurrealExecutor<C> for SurrealTransaction<C> {
    fn query<'client>(&'client self, query: impl Into<Cow<'client, str>>) -> Query<'client, C> {
        self.tx.query(query)
    }

    fn select<O>(&self, resource: impl IntoResource<O>) -> Select<C, O> {
        self.tx.select(resource)
    }

    fn create<R>(&self, resource: impl CreateResource<R>) -> Create<C, R> {
        self.tx.create(resource)
    }

    fn insert<O>(&self, resource: impl IntoResource<O>) -> Insert<C, O> {
        self.tx.insert(resource)
    }

    fn upsert<O>(&self, resource: impl IntoResource<O>) -> Upsert<C, O> {
        self.tx.upsert(resource)
    }

    fn update<O>(&self, resource: impl IntoResource<O>) -> Update<C, O> {
        self.tx.update(resource)
    }
    
    fn delete<O>(&self, resource: impl IntoResource<O>) -> Delete<C, O> {
        self.tx.delete(resource)
    }
}

impl<C: Connection> SurrealDatabase<C> {
    pub fn new(db: Surreal<C>) -> Self {
        Self { db }
    }

    pub async fn begin(&self) -> surrealdb::Result<SurrealTransaction<C>> {
        let tx = self.db.clone().begin().await?;
        Ok(SurrealTransaction { tx })
    }
}

impl<C: Connection> SurrealTransaction<C> {
    pub async fn commit(self) -> surrealdb::Result<()> {
        self.tx.commit().await?;
        Ok(())
    }

    pub async fn cancel(self) -> surrealdb::Result<()> {
        self.tx.cancel().await?;
        Ok(())
    }
}

pub struct SurrealdbAdaptor {

}

#[async_trait]
impl AccountRepositoryBlueprint for SurrealdbAdaptor {
    async fn create_account(
        &self,
        account: AccountModel<Created>,
    ) -> Result<AccountModel<Saved>, FormaError> {
        
    }
}