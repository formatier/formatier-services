use forma_core::domain::entities::{
    FormaError, FormaErrorApp, FormaErrorConverter, FormaErrorExt, FormaErrorExternalService,
    FormaErrorReqwestExt,
};
use forma_schema::{
    Request, ResultReply,
    auth_service::{AppAuthenticateV1Reply, AppAuthenticateV1Request},
};
use url::Url;

async fn into_result<T>(res: reqwest::Response) -> Result<T, FormaError>
where
    T: for<'de> serde::Deserialize<'de>,
{
    res.json::<ResultReply<_>>()
        .await
        .map_forma_err(
            FormaErrorApp::InvalidType,
            "cannot parse response to ResultReply",
        )?
        .into()
}

#[async_trait::async_trait]
pub trait AuthServiceBlueprint {
    fn get_url(&self) -> Url;
    fn get_client(&self) -> &reqwest::Client;

    async fn app_authenticate_v1(
        &self,
        access_token: &str,
    ) -> Result<AppAuthenticateV1Reply, FormaError> {
        let client = self.get_client();
        let mut url = self.get_url();
        url.set_path("/auth/middleware");

        let res = client
            .post(url)
            .body(
                Request {
                    data: AppAuthenticateV1Request {
                        access_token: access_token.into(),
                    },
                }
                .into_string()?,
            )
            .send()
            .await
            .map_reqwest_forma_err(
                FormaErrorExternalService::ConnectionError,
                "cannot connect to auth_service",
            )
            .with_service_breaker_request()?;

        into_result(res).await
    }
}

pub trait ServiceBlueprint: AuthServiceBlueprint {}
