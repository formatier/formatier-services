//! JWT issuance, parsing, and session-token lifecycle services.
use std::sync::Arc;

use crate::domain::entities::{
    AccessTokenClaims, Audience, InitializationTokenClaims, Issuer,
    RefreshTokenClaims, SessionModelPairs, TokenClaims,
};
use chrono::{DateTime, TimeDelta, Utc};
use forma_core::domain::entities::{FormaError, FormaErrorAuth, FormaErrorConverter, SavedModel};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        blueprints::{TokenCacheBlueprint, TokenRepositoryBlueprint},
        entities::TokenType,
    },
    value_objects,
};

/// Coordinates token creation and session state across persistence and cache layers.
pub struct TokenService {
    token_repository: Arc<dyn TokenRepositoryBlueprint + Send + Sync>,
    token_cache: Arc<dyn TokenCacheBlueprint + Send + Sync>,

    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    algorithm_config: Validation,
}

impl TokenService {
    /// Creates a token service from its persistence and cache dependencies.
    ///
    /// # Arguments
    /// * `token_repository` - Persistence operations for sessions.
    /// * `token_cache` - Cache operations for active token state.
    ///
    /// # Returns
    /// A token service configured with the application's token secret.
    pub fn new(
        token_repository: Arc<dyn TokenRepositoryBlueprint + Send + Sync>,
        token_cache: Arc<dyn TokenCacheBlueprint + Send + Sync>,
    ) -> Self {
        let encoding_key = EncodingKey::from_base64_secret(&value_objects::TOKEN_SECRET).unwrap();
        let decoding_key = DecodingKey::from_base64_secret(&value_objects::TOKEN_SECRET).unwrap();
        let algorithm_config = Validation::new(Algorithm::RS512);

        Self {
            token_repository,
            token_cache,

            encoding_key,
            decoding_key,
            algorithm_config,
        }
    }

    /// Encodes claims into a signed JWT with the requested token type.
    ///
    /// # Arguments
    /// * `claims` - Claims to serialize into the token.
    /// * `typ` - Token type recorded in the JWT header.
    ///
    /// # Returns
    /// The encoded JWT string.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if token encoding fails.
    fn encode_token<T: Serialize>(
        &self,
        claims: TokenClaims<T>,
        typ: TokenType,
    ) -> Result<String, FormaError> {
        let mut header = Header::new(self.algorithm_config.algorithms[0]);
        header.typ = Some(typ.to_string());

        let token = encode(&header, &claims, &self.encoding_key)
            .map_forma_err(FormaErrorAuth::TokenInvalid, "cannot encode token")?;
        Ok(token)
    }

    /// Decodes and validates a JWT and checks its declared token type.
    ///
    /// # Arguments
    /// * `token` - Encoded JWT to validate.
    /// * `typ` - Expected token type.
    ///
    /// # Returns
    /// The validated token claims.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if decoding, validation, or token-type checking
    /// fails.
    fn decode_token<T: for<'de> serde::Deserialize<'de>>(
        &self,
        token: &str,
        typ: TokenType,
    ) -> Result<TokenClaims<T>, FormaError> {
        let decoded = decode(token, &self.decoding_key, &self.algorithm_config)
            .map_forma_err(FormaErrorAuth::TokenInvalid, "cannot parse token")?;

        if decoded.header.typ.ok_or(FormaError::new(
            FormaErrorAuth::TokenInvalid,
            "invalid token type",
        ))? != typ.to_string()
        {
            return Err(FormaError::new(
                FormaErrorAuth::TokenInvalid,
                "invalid token type",
            ));
        }

        Ok(decoded.claims)
    }

    /// Creates a session and initializes its cache entry.
    ///
    /// # Arguments
    /// * `account_id` - Identifier of the account owning the session.
    /// * `user_agent` - Versioned user-agent information to persist.
    ///
    /// # Returns
    /// The new session's database identifier.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if metadata creation, persistence, or cache
    /// initialization fails.
    pub async fn create_session(
        &self,
        account_id: &str,
        user_agent: LatestSession,
    ) -> Result<ObjectId, FormaError> {
        let session_metadata = self
            .token_repository
            .create_session_metadata(account_id)
            .await?;
        let session = self
            .token_repository
            .create_session(
                SessionModelPairs {
                    data: user_agent.into(),
                    metadata: session_metadata,
                }
                .into(),
            )
            .await?;

        self.token_cache
            .create_token_data(&session.id().to_hex())
            .await?;

        Ok(session.id())
    }

    /// Issues access and refresh JWTs for a session.
    ///
    /// # Arguments
    /// * `session_id` - Identifier of the session.
    /// * `account_id` - Identifier of the account.
    /// * `user_id` - Identifier of the user.
    /// * `email` - Email claim associated with the user.
    /// * `scope` - Permissions included in the access token.
    ///
    /// # Returns
    /// A tuple containing the access token followed by the refresh token.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if either token cannot be encoded.
    pub async fn issue_tokens(
        &self,
        session_id: &str,
        account_id: &str,
        user_id: &str,
        email: &str,
        scope: &[&str],
    ) -> Result<(String, String), FormaError> {
        let access_token = self.encode_token(
            TokenClaims {
                issuer: Issuer::Formatier,
                audience: [
                    Audience::FormatierAPI,
                    Audience::FormatierBucketServer,
                    Audience::FormatineServer,
                    Audience::FormatierMailServer,
                ]
                .into(),
                expiration_time: (Utc::now() + TimeDelta::days(4)).into(),
                issue_at: Utc::now().into(),
                claims: AccessTokenClaims {
                    session_id: session_id.into(),
                    account_id: account_id.into(),
                    email: email.into(),
                    user_id: user_id.into(),
                    scope: scope.into_iter().map(|v| v.to_string()).collect(),
                },
            },
            TokenType::Access,
        )?;

        let refresh_token = self.encode_token(
            TokenClaims {
                issuer: Issuer::Formatier,
                audience: [
                    Audience::FormatierAPI,
                    Audience::FormatierBucketServer,
                    Audience::FormatineServer,
                    Audience::FormatierMailServer,
                ]
                .into(),
                expiration_time: (Utc::now() + TimeDelta::days(4)).into(),
                issue_at: Utc::now().into(),
                claims: RefreshTokenClaims {
                    session_id: session_id.into(),
                },
            },
            TokenType::Refresh,
        )?;

        Ok((access_token, refresh_token))
    }

    /// Issues an initialization token for an account.
    ///
    /// # Arguments
    /// * `account_id` - Identifier of the account being initialized.
    ///
    /// # Returns
    /// The encoded initialization token.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if the token cannot be encoded.
    pub fn issue_initialization_token(&self, account_id: &str) -> Result<String, FormaError> {
        let initialization_token = self.encode_token(
            TokenClaims {
                issuer: Issuer::Formatier,
                audience: [Audience::FormatierAPI].into(),
                expiration_time: (Utc::now() + TimeDelta::days(4)).into(),
                issue_at: Utc::now().into(),
                claims: InitializationTokenClaims {
                    account_id: account_id.into(),
                },
            },
            TokenType::Initialization,
        )?;

        Ok(initialization_token)
    }

    /// Parses a token and reports whether its expiration time has passed.
    ///
    /// # Arguments
    /// * `token` - Encoded token to parse.
    /// * `typ` - Expected token type.
    ///
    /// # Returns
    /// A pair of `(is_expired, claims)`.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if token decoding or validation fails.
    pub async fn parse_token<T: for<'de> Deserialize<'de>>(
        &self,
        token: &str,
        typ: TokenType,
    ) -> Result<(bool, TokenClaims<T>), FormaError> {
        let claims = self.decode_token(token, typ)?;

        let expiration_time: DateTime<Utc> = claims.expiration_time.clone().into();

        if expiration_time < Utc::now() {
            return Ok((true, claims));
        }

        Ok((false, claims))
    }

    /// Checks whether a session's token remains available.
    ///
    /// If no active cache entry exists, the repository is checked and the cache
    /// is initialized when the session is still persisted.
    ///
    /// # Arguments
    /// * `session_id` - Identifier of the session to check.
    ///
    /// # Returns
    /// `true` if the session is available; otherwise, `false`.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if cache or persistence access fails.
    pub async fn check_token_status(&self, session_id: &str) -> Result<bool, FormaError> {
        if self.token_cache.is_token_available(session_id).await? {
            Ok(true)
        } else {
            if self
                .token_repository
                .is_session_available(session_id)
                .await?
            {
                self.token_cache.create_token_data(session_id).await?;
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    /// Refreshes a session and reinitializes its token-cache entry.
    ///
    /// # Arguments
    /// * `session_id` - Identifier of the session to refresh.
    ///
    /// # Returns
    /// `()` when both operations succeed.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if persistence or cache updates fail.
    pub async fn refresh_tokens(&self, session_id: &str) -> Result<(), FormaError> {
        self.token_repository.refresh_session(session_id).await?;
        self.token_cache.create_token_data(session_id).await?;
        Ok(())
    }

    /// Deletes a session and removes its token-cache entry.
    ///
    /// # Arguments
    /// * `session_id` - Identifier of the session to revoke.
    ///
    /// # Returns
    /// `()` when both operations succeed.
    ///
    /// # Errors
    /// Returns a [`FormaError`] if persistence deletion or cache removal fails.
    pub async fn revoke_session(&self, session_id: &str) -> Result<(), FormaError> {
        self.token_repository.delete_session(session_id).await?;
        self.token_cache.remove_token(session_id).await?;
        Ok(())
    }
}
