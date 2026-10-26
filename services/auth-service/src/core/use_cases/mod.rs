//! Application workflows coordinating OAuth, persistence, and token services.
use std::{collections::HashMap, sync::Arc};

use crate::domain::entities::{
    AccessTokenClaims, Account, AccountMetadata, AccountModel, AccountModelPairs, AccountProvider, Provider, ProviderIdent, RefreshTokenClaims, TokenClaims,
};
use forma_core::domain::entities::{FormaError, FormaErrorAuth, Saved, SavedModel};

use crate::{
    core::services::TokenService,
    domain::{
        blueprints::{OAuthBlueprint, RepositoryBlueprint, TokenCacheBlueprint},
        entities::TokenType,
    },
};

/// Coordinates authentication use cases across provider and storage adapters.
pub struct UseCase {
    google_oauth: Arc<dyn OAuthBlueprint + Send + Sync>,
    github_oauth: Arc<dyn OAuthBlueprint + Send + Sync>,
    repository: Arc<dyn RepositoryBlueprint + Send + Sync>,
    token_service: TokenService,
}

impl UseCase {
    /// Creates a use case with OAuth, repository, and token-cache dependencies.
    pub fn new(
        google_oauth: Arc<dyn OAuthBlueprint + Send + Sync>,
        github_oauth: Arc<dyn OAuthBlueprint + Send + Sync>,
        repository: Arc<dyn RepositoryBlueprint + Send + Sync>,
        token_cache: Arc<dyn TokenCacheBlueprint + Send + Sync>,
    ) -> Self {
        Self {
            google_oauth: google_oauth.clone(),
            github_oauth: github_oauth.clone(),
            repository: repository.clone(),

            token_service: TokenService::new(repository.clone(), token_cache.clone()),
        }
    }

    /// Signs in or begins account initialization through an OAuth provider.
    pub async fn signin_by_oauth_provider(
        &self,
        provider: ProviderIdent,
        code: &str,
        user_agent: &str,
    ) -> Result<(String, Option<String>), FormaError> {
        let oauth_user_info = match provider {
            ProviderIdent::Google => {
                let token = self.google_oauth.exchange_code(code).await?;
                self.google_oauth.get_authorized_user(&token).await?
            }
            ProviderIdent::Github => {
                let token = self.github_oauth.exchange_code(code).await?;
                self.github_oauth.get_authorized_user(&token).await?
            }
        };

        let account = self
            .repository
            .get_account_by_oauth_provider(provider, &oauth_user_info.provider_id)
            .await?;

        let account = match account {
            Some(account) => account,
            None => {
                self.repository
                    .create_account(
                        AccountModelPairs {
                            data: Account {
                                email: oauth_user_info.email,
                                providers: HashMap::from([(
                                    provider,
                                    AccountProvider {
                                        provider_id: oauth_user_info.provider_id,
                                    },
                                )]),
                                ..Default::default()
                            }
                            .into(),
                            metadata: AccountMetadata::default(),
                        }
                        .into(),
                    )
                    .await?
            }
        };

        let account_id = account.id().to_hex();
        let account_metadata = account.metadata;
        let (account, _) = account.data.migrate();

        if let Some(user_id) = account_metadata.user_id {
            let user_agent =
                woothee::parser::Parser::default()
                    .parse(user_agent)
                    .ok_or(FormaError::new(
                        FormaErrorAuth::InvalidUserAgent,
                        "cannot parse user-agent",
                    ))?;
            let session_id = self
                .token_service
                .create_session(&account_id, user_agent.into())
                .await?;
            let (access_token, refresh_token) = self
                .token_service
                .issue_tokens(
                    &session_id.to_hex(),
                    &account_id,
                    &user_id.to_hex(),
                    &account.email,
                    &["all"],
                )
                .await?;

            Ok((access_token, Some(refresh_token)))
        } else {
            let initialization_token =
                self.token_service.issue_initialization_token(&account_id)?;
            Ok((initialization_token, None))
        }
    }

    /// Decodes an access token and checks its session status.
    pub async fn decode_token(
        &self,
        access_token: &str,
    ) -> Result<TokenClaims<AccessTokenClaims>, FormaError> {
        let (access_token_expired, access_token_claims) = self
            .token_service
            .parse_token::<AccessTokenClaims>(access_token, TokenType::Access)
            .await?;

        if access_token_expired {
            Err(FormaError::new(
                FormaErrorAuth::TokenExpired,
                "refresh token expired",
            ))
        } else {
            if self
                .token_service
                .check_token_status(&access_token_claims.claims.session_id)
                .await?
            {
                Err(FormaError::new(
                    FormaErrorAuth::TokenRevoked,
                    "token revoked",
                ))
            } else {
                Ok(access_token_claims)
            }
        }
    }

    /// Validates a refresh token and issues replacement access and refresh tokens.
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<(String, String), FormaError> {
        let (expired, token_claims) = self
            .token_service
            .parse_token::<RefreshTokenClaims>(refresh_token, TokenType::Refresh)
            .await?;
        if expired {
            return Err(FormaError::new(
                FormaErrorAuth::TokenExpired,
                "refresh token expired",
            ));
        }

        self.token_service.refresh_tokens(&token_claims.claims.session_id);

        self.token_service.issue_tokens(session_id, account_id, user_id, email, scope)
    }

    /// Retrieves an account by identifier.
    pub async fn get_account(&self, account_id: &str) -> Result<AccountModel<Saved>, FormaError> {
        let account_model = self.repository.get_account(account_id).await?;
        account_model
    }
}
