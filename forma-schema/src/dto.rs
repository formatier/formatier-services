use forma_core::domain::entities::{
    ChronoTime, FormaError, FormaErrorApp, FormaErrorConverter, Timestamp,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ResultReply<T> {
    Success(T),
    Error(FormaError),
}

impl<T> ResultReply<T> {
    pub fn is_success(&self) -> bool {
        matches!(self, ResultReply::Success(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, ResultReply::Error(_))
    }

    pub fn into_result(self) -> Result<T, FormaError> {
        match self {
            ResultReply::Success(value) => Ok(value),
            ResultReply::Error(err) => Err(err),
        }
    }
}

impl<T> Into<Result<T, FormaError>> for ResultReply<T> {
    fn into(self) -> Result<T, FormaError> {
        self.into_result()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Request<T> {
    #[serde(flatten)]
    pub data: T,
}

impl<T> Request<T>
where
    T: Serialize,
{
    pub fn into_string(self) -> Result<String, FormaError> {
        let data = serde_json::to_string(&self.data)
            .map_forma_err(FormaErrorApp::InvalidType, "failed to serialize data")?;
        Ok(data)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Reply<T> {
    #[serde(flatten)]
    pub data: T,
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Metadata {
    pub timestamp: Timestamp<ChronoTime>,
}

#[derive(Serialize, Deserialize)]
pub struct SagaRequest<T> {
    #[serde(flatten)]
    pub data: T,
    pub metadata: SagaTransactionMetadata,
}

#[derive(Serialize, Deserialize)]
pub struct SagaReply<T> {
    #[serde(flatten)]
    pub data: T,
    pub metadata: SagaTransactionMetadata,
}

#[derive(Serialize, Deserialize)]
pub struct SagaTransactionMetadata {
    id: Uuid,
    create_at: Timestamp<ChronoTime>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct HealthzCheck {
    pub status: HealthzStatus,
    pub down_list: Option<Box<[Box<str>]>>
}

#[derive(Serialize, Deserialize, Default)]
pub enum HealthzStatus {
    #[default]
    Ok,
    Down,
}