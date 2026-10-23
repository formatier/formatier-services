//! Generic request, reply, saga, and health-check data transfer objects.
//!
//! These types centralize the JSON wire shapes shared by service boundaries.
use super::{ChronoTime, FormaError, FormaErrorApp, FormaErrorConverter, Timestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
/// Represents a successful response or a serialized application error.
pub enum ResultReply<T> {
    /// Carries the successful response value.
    Success(T),
    /// Carries the application error returned by the operation.
    Error(FormaError),
}

impl<T> ResultReply<T> {
    /// Reports whether this reply contains a successful value.
    ///
    /// # Returns
    /// `true` for [`ResultReply::Success`] and `false` otherwise.
    pub fn is_success(&self) -> bool {
        matches!(self, ResultReply::Success(_))
    }

    /// Reports whether this reply contains an application error.
    ///
    /// # Returns
    /// `true` for [`ResultReply::Error`] and `false` otherwise.
    pub fn is_error(&self) -> bool {
        matches!(self, ResultReply::Error(_))
    }

    /// Converts this reply into the corresponding standard result.
    ///
    /// # Returns
    /// The success value or the contained [`FormaError`].
    pub fn into_result(self) -> Result<T, FormaError> {
        match self {
            ResultReply::Success(value) => Ok(value),
            ResultReply::Error(err) => Err(err),
        }
    }
}

impl<T> Into<Result<T, FormaError>> for ResultReply<T> {
    /// Converts this reply into the corresponding standard result.
    ///
    /// # Returns
    /// The success value or the contained [`FormaError`].
    fn into(self) -> Result<T, FormaError> {
        self.into_result()
    }
}

#[derive(Serialize, Deserialize)]
/// Wraps request data using the shared flattened wire representation.
pub struct Request<T> {
    #[serde(flatten)]
    /// The request payload.
    pub data: T,
}

impl<T> Request<T>
where
    T: Serialize,
{
    /// Serializes the request payload as JSON.
    ///
    /// # Returns
    /// The serialized JSON string.
    ///
    /// # Errors
    /// Returns a [`FormaError`] when the payload cannot be serialized.
    pub fn into_string(self) -> Result<String, FormaError> {
        let data = serde_json::to_string(&self.data)
            .map_forma_err(FormaErrorApp::InvalidType, "failed to serialize data")?;
        Ok(data)
    }
}

#[derive(Serialize, Deserialize)]
/// Wraps response data together with response metadata.
pub struct Reply<T> {
    #[serde(flatten)]
    /// The response payload.
    pub data: T,
    /// Metadata associated with this response.
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, Default)]
/// Provides common metadata attached to a response.
pub struct Metadata {
    /// The timestamp at which the response metadata was created.
    pub timestamp: Timestamp<ChronoTime>,
}

#[derive(Serialize, Deserialize)]
/// Wraps request data and its saga transaction metadata.
pub struct SagaRequest<T> {
    #[serde(flatten)]
    /// The request payload.
    pub data: T,
    /// Identifies and timestamps the saga transaction.
    pub metadata: SagaTransactionMetadata,
}

#[derive(Serialize, Deserialize)]
/// Wraps response data and its saga transaction metadata.
pub struct SagaReply<T> {
    #[serde(flatten)]
    /// The response payload.
    pub data: T,
    /// Identifies and timestamps the saga transaction.
    pub metadata: SagaTransactionMetadata,
}

#[derive(Serialize, Deserialize)]
/// Identifies a saga transaction and records when it was created.
pub struct SagaTransactionMetadata {
    /// Unique identifier for the saga transaction.
    id: Uuid,
    /// Creation timestamp for the saga transaction.
    create_at: Timestamp<ChronoTime>,
}

#[derive(Serialize, Deserialize, Default)]
/// Reports the health state of a service and any components that are down.
pub struct HealthzCheck {
    /// Overall health state.
    pub status: HealthzStatus,
    /// Names of components that are unavailable, when any are reported.
    pub down_list: Option<Box<[Box<str>]>>,
}

#[derive(Serialize, Deserialize, Default)]
/// Represents the overall result of a health check.
pub enum HealthzStatus {
    /// All checked components are healthy.
    #[default]
    Ok,
    /// At least one checked component is unavailable.
    Down,
}
