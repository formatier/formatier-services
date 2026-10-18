use std::{collections::HashMap, sync::Arc};

use forma_core::domain::entities::{FormaError, FormaErrorAuth, Saved};
use forma_schema::auth_service::{
    AccessTokenClaimsV1, AccountV1, AccountV1Metadata, AccountV1Model, AccountV1ModelPairs,
    AccountV1Provider, ProviderV1, RefreshTokenClaimsV1, TokenClaimsV1,
};

use crate::{
    core::services::TokenService,
    domain::{
        blueprints::{OAuthBlueprint, RepositoryBlueprint, TokenCacheBlueprint},
        entities::TokenType,
    },
};

pub struct UseCase {
    google_oauth: Arc<dyn OAuthBlueprint + Send + Sync>,
    github_oauth: Arc<dyn OAuthBlueprint + Send + Sync>,
    repository: Arc<dyn RepositoryBlueprint + Send + Sync>,
    token_service: TokenService,
}

impl UseCase {
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

    pub async fn signin_by_oauth_provider_v1(
        &self,
        provider: ProviderV1,
        code: &str,
        user_agent: &str,
    ) -> Result<(String, Option<String>), FormaError> {
        let oauth_user_info = match provider {
            ProviderV1::Google => {
                let token = self.google_oauth.exchange_code(code).await?;
                self.google_oauth.get_authorized_user(&token).await?
            }
            ProviderV1::Github => {
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
                    .create_account(AccountV1ModelPairs {
                        data: AccountV1 {
                            email: oauth_user_info.email,
                            providers: HashMap::from([(
                                provider,
                                AccountV1Provider {
                                    provider_id: oauth_user_info.provider_id,
                                },
                            )]),
                            ..Default::default()
                        }
                        .into(),
                        metadata: AccountV1Metadata::default(),
                    })
                    .await?
            }
        };

        let account_id = account.id().to_hex();
        let account_metadata = account.metadata;

        if let Some(user_id) = account_metadata.user_id {
            let user_agent =
                woothee::parser::Parser::default()
                    .parse(user_agent)
                    .ok_or(FormaError::new(
                        FormaErrorAuth::InvalidUserAgent,
                        "cannot parse user-agent",
                    ))?;
            let (access_token, refresh_token) = self
                .token_service
                .issue_tokens_and_create_session(
                    &account_id,
                    &user_id.to_hex(),
                    &account.data.email,
                    &["all"],
                    user_agent.into(),
                )
                .await?;

            Ok((access_token, Some(refresh_token)))
        } else {
            let initialization_token =
                self.token_service.issue_initialization_token(&account_id)?;
            Ok((initialization_token, None))
        }
    }

    pub async fn decode_token_v1(
        &self,
        access_token: &str,
    ) -> Result<TokenClaimsV1<AccessTokenClaimsV1>, FormaError> {
        let (access_token_expired, access_token_claims) = self
            .token_service
            .parse_token::<AccessTokenClaimsV1>(access_token, TokenType::Access)
            .await?;

        if access_token_expired {
            Err(FormaError::new(
                FormaErrorAuth::TokenExpired,
                "refresh token expired",
            ));
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

    pub async fn refresh_token_v1(
        &self,
        refresh_token: &str,
    ) -> Result<(String, String), FormaError> {
        
    }

    pub async fn get_account_v1(
        &self,
        account_id: &str,
    ) -> Result<AccountV1Model<Saved>, FormaError> {
        let account_model = self.repository.get_account(account_id).await?;
        account_model
    }
}
