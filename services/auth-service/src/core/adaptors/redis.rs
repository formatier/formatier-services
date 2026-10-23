//! Redis-backed implementation of the authentication token cache.
use async_trait::async_trait;
use bb8::{Pool, PooledConnection};
use chrono::TimeDelta;
use forma_core::domain::entities::{
    FormaError, FormaErrorConverter, FormaErrorDatabase, FormaErrorExternalService, FormaErrorRedisExt,
};
use redis::AsyncCommands;

use crate::domain::blueprints::TokenCacheBlueprint;

/// Stores and checks authentication token state in Redis.
pub struct RedisAdaptor {
    redis_conn: Pool<redis::Client>,
}

impl RedisAdaptor {
    /// Creates an adapter using the provided Redis connection pool.
    ///
    /// # Arguments
    /// * `redis_conn` - Pool used to acquire Redis connections.
    ///
    /// # Returns
    /// A Redis token-cache adapter.
    pub fn new(redis_conn: Pool<redis::Client>) -> Self {
        Self { redis_conn }
    }

    /// Acquires a pooled Redis connection.
    ///
    /// # Returns
    /// A connection borrowed from the adapter's pool.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the pool cannot provide a connection.
    async fn get_conn(&self) -> Result<PooledConnection<'_, redis::Client>, FormaError> {
        self.redis_conn
            .get()
            .await
            .map_forma_err(FormaErrorExternalService::PoolError, "cannot get pool")
    }
}

#[async_trait]
impl TokenCacheBlueprint for RedisAdaptor {
    /// Creates an active token-cache entry and gives it a two-hour expiry.
    ///
    /// # Arguments
    /// * `session_id` - Identifier of the session whose token state to create.
    ///
    /// # Returns
    /// `()` when the cache entry is created successfully.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the connection or any Redis operation fails.
    async fn create_token_data(&self, session_id: &str) -> Result<(), FormaError> {
        let mut conn = self.get_conn().await?;
        let key = format!("token:token:{}", session_id);

        let ok: bool = conn.set(&key, 1).await.map_redis_forma_err(
            FormaErrorExternalService::ConnectionError,
            "cannot connect to redis",
        )?;

        if !ok {
            return Err(FormaError::new(
                FormaErrorDatabase::InsertError,
                "cannot insert token data to cache",
            ));
        }

        let ok: bool = conn
            .expire(&key, TimeDelta::hours(2).num_seconds())
            .await
            .map_redis_forma_err(
                FormaErrorExternalService::ConnectionError,
                "cannot connect to redis",
            )?;

        if !ok {
            let ok: bool = conn.del(&key).await.map_redis_forma_err(
                FormaErrorExternalService::ConnectionError,
                "cannot connect to redis",
            )?;

            return Err(FormaError::new(
                FormaErrorDatabase::InsertError,
                if ok {
                    "cannot insert token data to cache"
                } else {
                    "cannot rollback operation"
                },
            ));
        }

        Ok(())
    }

    /// Marks a session's token-cache entry as inactive.
    ///
    /// # Arguments
    /// * `session_id` - Identifier of the session to deactivate.
    ///
    /// # Returns
    /// `()` when the cache update succeeds.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the connection or Redis update fails.
    async fn remove_token(&self, session_id: &str) -> Result<(), FormaError> {
        let mut conn = self.get_conn().await?;
        let key = format!("token:token:{}", session_id);

        let ok: bool = conn.set(key, 0).await.map_redis_forma_err(
            FormaErrorExternalService::ConnectionError,
            "cannot connect to redis",
        )?;

        if !ok {
            return Err(FormaError::new(
                FormaErrorDatabase::UpdateError,
                "cannot invoke token",
            ));
        }

        Ok(())
    }

    /// Checks whether a session has an active token-cache entry.
    ///
    /// # Arguments
    /// * `session_id` - Identifier of the session to check.
    ///
    /// # Returns
    /// `true` if the cached value is active; otherwise, `false`.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the connection or Redis lookup fails.
    async fn is_token_available(&self, session_id: &str) -> Result<bool, FormaError> {
        let mut conn = self.get_conn().await?;
        let key = format!("token:token:{}", session_id);

        let (val, ok): (i8, bool) = conn.get(key).await.map_redis_forma_err(
            FormaErrorExternalService::ConnectionError,
            "cannot connect to redis",
        )?;

        if !ok {
            return Err(FormaError::new(
                FormaErrorDatabase::UpdateError,
                "cannot get token data from cache",
            ));
        }

        Ok(val == 1)
    }
}
