//! Shared domain entities, persistence markers, timestamps, and error types.
//! Shared types and persistence helpers for Formatier services.
use std::{
    error::Error,
    fmt::{Debug, Display},
    marker::PhantomData,
};

use axum::{http::StatusCode, response::IntoResponse};
use chrono::{DateTime, Utc};
use mongodb::bson::{self};
use serde::{Deserialize, Serialize, de::Visitor};
use surrealdb::types::{self, Kind, SurrealValue};

use crate::inline_mod;

inline_mod!(mongo);
inline_mod!(surreal);
inline_mod!(dto);

#[derive(Clone)]
/// Marks a persistence value that has not yet been assigned a database ID.
pub struct Created;
#[derive(Clone)]
/// Marks a persistence value that has a database ID.
pub struct Saved;

/// Describes whether a persistence state requires a database identifier.
pub trait TableState {
    /// Whether values in this state must contain an identifier.
    const HAS_ID: bool;
}
impl TableState for Created {
    const HAS_ID: bool = false;
}
impl TableState for Saved {
    const HAS_ID: bool = true;
}

#[derive(Clone)]
/// Selects BSON date-time serialization for a [`Timestamp`].
pub struct BsonTime;
#[derive(Clone)]
/// Selects Chrono timestamp serialization for a [`Timestamp`].
pub struct ChronoTime;
#[derive(Clone)]
/// Selects SurrealDB date-time serialization for a [`Timestamp`].
pub struct SurrealTime;

/// Deserializes an integer millisecond timestamp into a Chrono date-time.
impl<'de> Visitor<'de> for ChronoTime {
    /// The date-time value produced by this visitor.
    type Value = chrono::DateTime<Utc>;

    /// Converts a millisecond timestamp into a UTC date-time.
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        chrono::DateTime::from_timestamp_millis(v)
            .ok_or(E::invalid_length(i32::MAX as usize, &self))
    }

    /// Describes the value expected by this visitor.
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "cannot parse timestamp")
    }
}

#[derive(Clone)]
/// Stores a millisecond timestamp with a type-level serialization format.
pub struct Timestamp<T>(TimestampInner, PhantomData<T>);
#[derive(Clone)]
/// Stores the timestamp value shared by all serialization representations.
struct TimestampInner {
    /// Milliseconds since the Unix epoch.
    timestamp: i64,
}

impl<T> Default for Timestamp<T> {
    /// Creates a timestamp for the current UTC time.
    fn default() -> Self {
        Self(
            TimestampInner {
                timestamp: Utc::now().timestamp_millis(),
            },
            PhantomData,
        )
    }
}

impl<T> Timestamp<T> {
    /// Constructs a timestamp from milliseconds since the Unix epoch.
    pub fn from_timestamp(timestamp: i64) -> Result<Self, FormaError> {
        Ok(Self(
            TimestampInner {
                timestamp: chrono::DateTime::from_timestamp_millis(timestamp)
                    .ok_or(FormaError::new(
                        FormaErrorApp::InvalidTimestamp,
                        "cannot parse timestamp",
                    ))?
                    .timestamp_millis(),
            },
            PhantomData,
        ))
    }
}

impl<T> From<bson::DateTime> for Timestamp<T> {
    /// Converts a BSON date-time to the shared millisecond representation.
    fn from(value: bson::DateTime) -> Self {
        Timestamp::<T>(
            TimestampInner {
                timestamp: value.timestamp_millis(),
            },
            PhantomData,
        )
    }
}

impl<T> Into<bson::DateTime> for Timestamp<T> {
    /// Converts this timestamp to a BSON date-time.
    fn into(self) -> bson::DateTime {
        bson::DateTime::from_millis(self.0.timestamp)
    }
}

impl<T> From<DateTime<Utc>> for Timestamp<T> {
    /// Converts a UTC date-time to the shared millisecond representation.
    fn from(value: DateTime<Utc>) -> Self {
        Timestamp::<T>(
            TimestampInner {
                timestamp: value.timestamp_millis(),
            },
            PhantomData,
        )
    }
}

impl<T> Into<chrono::DateTime<Utc>> for Timestamp<T> {
    /// Converts this timestamp to a UTC date-time.
    fn into(self) -> chrono::DateTime<Utc> {
        chrono::DateTime::from_timestamp_millis(self.0.timestamp).unwrap()
    }
}

impl Serialize for Timestamp<BsonTime> {
    /// Serializes this timestamp using BSON date-time encoding.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bson_datetime = bson::DateTime::from_millis(self.0.timestamp);
        bson_datetime.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Timestamp<BsonTime> {
    /// Deserializes a BSON date-time into a timestamp.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let time = bson::DateTime::deserialize(deserializer)?;
        Ok(Self::from(time))
    }
}

impl Serialize for Timestamp<ChronoTime> {
    /// Serializes this timestamp as an integer millisecond value.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i64(self.0.timestamp)
    }
}

impl<'de> Deserialize<'de> for Timestamp<ChronoTime> {
    /// Deserializes an integer millisecond timestamp.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let timestamp = deserializer.deserialize_i64(ChronoTime)?;
        Ok(Self::from(timestamp))
    }
}

impl Serialize for Timestamp<SurrealTime> {
    /// Serializes this timestamp as an integer millisecond value.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i64(self.0.timestamp)
    }
}

impl<'de> Deserialize<'de> for Timestamp<SurrealTime> {
    /// Deserializes an integer millisecond timestamp.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let timestamp = deserializer.deserialize_i64(ChronoTime)?;
        Ok(Self::from(timestamp))
    }
}

impl SurrealValue for Timestamp<SurrealTime> {
    fn kind_of() -> Kind {
        Kind::Datetime
    }

    /// Converts the timestamp to a SurrealDB value.
    fn into_value(self) -> types::Value {
        let date_time = DateTime::from_timestamp_millis(self.0.timestamp).unwrap();
        date_time.into_value()
    }

    /// Decodes a SurrealDB value as a timestamp.
    fn from_value(value: types::Value) -> Result<Self, surrealdb::Error> {
        Ok(Self::from(DateTime::<Utc>::from_value(value)?))
    }
}

#[derive(thiserror::Error, Clone, Serialize, Deserialize, Default)]
#[error("{kind} :: {message}")]
/// Represents an application error and its optional internal diagnostics.
pub struct FormaError {
    #[source]
    /// Error category exposed to callers.
    pub kind: FormaErrorKind,
    /// More specific internal category inferred from the underlying failure.
    pub internal_kind: Option<FormaInternalErrorKind>,
    /// Human-readable error message.
    pub message: String,
    /// Optional structured details associated with the error.
    pub detail: Option<FormaErrorDetail>,
    #[serde(default)]
    /// Whether the error should be excluded from saga failure handling.
    pub non_saga_error: bool,

    #[serde(skip)]
    /// Diagnostics retained for logs but omitted from serialized responses.
    pub internal_info: Option<IntenalInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
/// Structured optional details attached to an application error.
pub enum FormaErrorDetail {
    /// Identifies an invalid or missing key.
    Key {
        /// The key associated with the error.
        key: String,
        /// Optional explanation of the key error.
        message: Option<String>,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Describes the operational severity of internal error information.
pub enum InternalLevel {
    /// Indicates an internal invariant or condition requiring immediate attention.
    Panic,
    /// Indicates a failure that may affect other instances in the cluster.
    ClusterException,
    /// Indicates a request or operation error.
    #[default]
    Error,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Stores diagnostic information about an underlying error.
pub struct IntenalInfo {
    /// Type or category of the underlying error.
    pub target_kind: Option<String>,
    /// Internal category assigned to the underlying error.
    pub internal_kind: FormaInternalErrorKind,
    /// Underlying error message.
    pub message: Option<String>,
    /// Optional source error message.
    pub stack_trace: Option<String>,
    /// Severity assigned to the underlying error.
    pub level: InternalLevel,
}

/// Formats the error and its internal diagnostics for debugging.
impl Debug for FormaError {
    /// Writes the serialized error payload and internal diagnostic detail.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "error payload: {}",
            serde_json::to_string_pretty(self).unwrap()
        )?;
        writeln!(
            f,
            "internal detail: {}",
            serde_json::to_string_pretty(&self.internal_info).unwrap()
        )
    }
}

impl FormaError {
    /// Creates an error from a kind and message.
    pub fn new<T: Into<FormaErrorKind>>(kind: T, message: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
            ..Default::default()
        }
    }

    /// Creates an error with optional structured details.
    pub fn new_with_detail<T: Into<FormaErrorKind>>(
        kind: T,
        message: &str,
        detail: Option<FormaErrorDetail>,
    ) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
            detail,
            ..Default::default()
        }
    }

    /// Maps this error kind to its HTTP status code.
    pub fn get_status(&self) -> StatusCode {
        match &self.kind {
            FormaErrorKind::FormaError(err) => match err {
                FormaErrorApp::InvalidType => StatusCode::BAD_REQUEST,
                FormaErrorApp::InvalidTimestamp => StatusCode::UNPROCESSABLE_ENTITY,
                FormaErrorApp::InvalidCredentials | FormaErrorApp::InvalidVersionNumber => {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
                FormaErrorApp::DataConflict => StatusCode::CONFLICT,
                FormaErrorApp::OptionNone => StatusCode::INTERNAL_SERVER_ERROR,
            },
            FormaErrorKind::AuthError(err) => match err {
                FormaErrorAuth::InternalEncryptionFail => StatusCode::INTERNAL_SERVER_ERROR,
                FormaErrorAuth::InvalidSignature => StatusCode::FORBIDDEN,
                FormaErrorAuth::TokenExpired
                | FormaErrorAuth::TokenInvalid
                | FormaErrorAuth::TokenRevoked => StatusCode::UNAUTHORIZED,
                FormaErrorAuth::InvalidUserAgent => StatusCode::IM_A_TEAPOT,
            },
            FormaErrorKind::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            FormaErrorKind::ExternalServiceError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            FormaErrorKind::OAuthError(err) => match err {
                FormaErrorOAuth::Unauthorized => StatusCode::UNAUTHORIZED,
                FormaErrorOAuth::NotFound => StatusCode::NOT_FOUND,
            },
            FormaErrorKind::CacheError(err) => match err {
                FormaErrorCache::CacheUnavailable => StatusCode::SERVICE_UNAVAILABLE,
                FormaErrorCache::IOError => StatusCode::INTERNAL_SERVER_ERROR,
                FormaErrorCache::Timeout => StatusCode::REQUEST_TIMEOUT,
            },
            FormaErrorKind::Internal | FormaErrorKind::Unhandled => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    /// Converts this error into an HTTP response.
    pub fn response(self) -> impl IntoResponse {
        let body = serde_json::to_string(&self).unwrap();
        axum::http::Response::builder()
            .status(self.get_status())
            .body(body)
            .unwrap()
    }
}

pub trait FormaErrorFlattener<T> {
    fn flatten_forma(self) -> Result<T, FormaError>;
}

impl<T> FormaErrorFlattener<T> for Result<Option<T>, FormaError> {
    fn flatten_forma(self) -> Result<T, FormaError> {
        self.transpose().ok_or_forma_error().flatten()
    }
}

impl<T> FormaErrorFlattener<T> for Option<Result<T, FormaError>> {
    fn flatten_forma(self) -> Result<T, FormaError> {
        self.ok_or_forma_error().flatten()
    }
}

/// Converts arbitrary errors into [`FormaError`] values.
pub trait FormaErrorConverter<T> {
    /// Maps an error to a public kind and fixed message.
    fn map_forma_err<K: Into<FormaErrorKind>>(
        self,
        kind: K,
        message: &str,
    ) -> Result<T, FormaError>;
    /// Maps an error to a public kind and attaches optional structured detail.
    fn map_forma_err_with_detail<K: Into<FormaErrorKind>, D: Into<Option<FormaErrorDetail>>>(
        self,
        kind: K,
        message: &str,
        detail: D,
    ) -> Result<T, FormaError>;
}

impl<T, E> FormaErrorConverter<T> for Result<T, E>
where
    E: Display + std::error::Error,
{
    /// Maps a failed result to a `FormaError` while preserving source diagnostics.
    fn map_forma_err<K: Into<FormaErrorKind>>(
        self,
        kind: K,
        message: &str,
    ) -> Result<T, FormaError> {
        self.map_err(|err| FormaError {
            kind: kind.into(),
            internal_kind: None,
            message: format!("{}", message),
            detail: None,
            non_saga_error: false,
            internal_info: Some(IntenalInfo {
                target_kind: None,
                internal_kind: FormaInternalErrorKind::Internal,
                message: Some(err.to_string()),
                stack_trace: err.source().map(|v| v.to_string()),
                level: InternalLevel::Error,
            }),
        })
    }

    /// Maps a failed result and includes optional structured details.
    fn map_forma_err_with_detail<K: Into<FormaErrorKind>, D: Into<Option<FormaErrorDetail>>>(
        self,
        kind: K,
        message: &str,
        detail: D,
    ) -> Result<T, FormaError> {
        self.map_err(|err| FormaError {
            kind: kind.into(),
            internal_kind: None,
            message: format!("{} {}", message, err.to_string()),
            detail: detail.into(),
            non_saga_error: false,
            internal_info: Some(IntenalInfo {
                target_kind: None,
                internal_kind: FormaInternalErrorKind::Internal,
                message: Some(err.to_string()),
                stack_trace: err.source().map(|v| v.to_string()),
                level: InternalLevel::Error,
            }),
        })
    }
}

/// Converts optional values into results using standard `FormaError` values.
pub trait FormaOptionConverter<T> {
    /// Converts `None` into a default option error.
    fn ok_or_forma_error(self) -> Result<T, FormaError>;

    /// Converts `None` into an option error with optional details.
    fn ok_or_forma_error_with_detail(
        self,
        detail: impl Into<Option<FormaErrorDetail>>,
    ) -> Result<T, FormaError>;
}

impl<T> FormaOptionConverter<T> for Option<T> {
    /// Converts `None` into a default option error.
    fn ok_or_forma_error(self) -> Result<T, FormaError> {
        self.ok_or_forma_error_with_detail(None)
    }

    /// Converts `None` into an option error with optional structured details.
    fn ok_or_forma_error_with_detail(
        self,
        detail: impl Into<Option<FormaErrorDetail>>,
    ) -> Result<T, FormaError> {
        self.ok_or(FormaError {
            kind: FormaErrorApp::OptionNone.into(),
            internal_kind: None,
            message: "option is none".into(),
            detail: detail.into(),
            non_saga_error: false,
            internal_info: None,
        })
    }
}

/// Adds common handling behavior to results containing [`FormaError`].
pub trait FormaErrorExt<T> {
    /// Raises internal diagnostic severity to panic level.
    fn with_service_breaker_request(self) -> Result<T, FormaError>;

    /// Marks an error so saga handling can treat it as non-saga-related.
    fn non_saga_error(self) -> Result<T, FormaError>;
}

impl<T> FormaErrorExt<T> for Result<T, FormaError> {
    /// Raises internal diagnostic severity to panic level.
    fn with_service_breaker_request(self) -> Result<T, FormaError> {
        self.map_err(|mut err| {
            err.internal_info = Some(err.internal_info.map_or_default(|mut internal_info| {
                internal_info.level = InternalLevel::Panic;
                internal_info
            }));
            err
        })
    }

    /// Marks an error as unrelated to saga failure handling.
    fn non_saga_error(self) -> Result<T, FormaError> {
        self.map_err(|mut err| {
            err.non_saga_error = true;
            err
        })
    }
}

/// Converts MongoDB errors into [`FormaError`] values.
pub trait FormaErrorMongoExt<T> {
    /// Maps a MongoDB error to the supplied public kind and message.
    fn map_mongo_forma_err<K: Into<FormaErrorKind>>(
        self,
        kind: K,
        message: &str,
    ) -> Result<T, FormaError>;
    /// Maps a MongoDB error and attaches optional structured details.
    fn map_mongo_forma_err_with_detail<K: Into<FormaErrorKind>>(
        self,
        kind: K,
        message: &str,
        detail: Option<FormaErrorDetail>,
    ) -> Result<T, FormaError>;
}

impl<T> FormaErrorMongoExt<T> for Result<T, mongodb::error::Error> {
    /// Maps a MongoDB error to the supplied public kind and message.
    fn map_mongo_forma_err<K: Into<FormaErrorKind>>(
        self,
        kind: K,
        message: &str,
    ) -> Result<T, FormaError> {
        self.map_mongo_forma_err_with_detail(kind, message, None)
    }

    /// Maps a MongoDB error and attaches optional structured details.
    fn map_mongo_forma_err_with_detail<K: Into<FormaErrorKind>>(
        self,
        kind: K,
        message: &str,
        detail: Option<FormaErrorDetail>,
    ) -> Result<T, FormaError> {
        self.map_err(move |err| {
            let intenal_info = expand_mongodb_error(err);

            FormaError {
                kind: kind.into(),
                internal_kind: Some(intenal_info.internal_kind),
                message: message.into(),
                detail,
                non_saga_error: false,
                internal_info: Some(intenal_info),
            }
        })
    }
}

/// Converts Redis errors into [`FormaError`] values.
pub trait FormaErrorRedisExt<T> {
    /// Maps a Redis error to a public kind and message.
    fn map_redis_forma_err<K: Into<FormaErrorKind>>(
        self,
        kind: K,
        message: &str,
    ) -> Result<T, FormaError>;
    /// Maps a Redis error and attaches optional structured details.
    fn map_redis_forma_err_with_detail<K: Into<FormaErrorKind>>(
        self,
        kind: K,
        message: &str,
        detail: Option<FormaErrorDetail>,
    ) -> Result<T, FormaError>;
}

impl<T> FormaErrorRedisExt<T> for Result<T, redis::RedisError> {
    /// Maps a Redis error to a public kind and message.
    fn map_redis_forma_err<K: Into<FormaErrorKind>>(
        self,
        kind: K,
        message: &str,
    ) -> Result<T, FormaError> {
        self.map_redis_forma_err_with_detail(kind, message, None)
    }

    /// Maps a Redis error and attaches optional structured details.
    fn map_redis_forma_err_with_detail<K: Into<FormaErrorKind>>(
        self,
        kind: K,
        message: &str,
        detail: Option<FormaErrorDetail>,
    ) -> Result<T, FormaError> {
        self.map_err(|err| {
            let internal_info = expand_redis_error(err);
            FormaError {
                kind: kind.into(),
                internal_kind: Some(internal_info.internal_kind),
                message: message.to_string(),
                detail,
                non_saga_error: false,
                internal_info: Some(internal_info),
            }
        })
    }
}

/// Converts Reqwest errors into [`FormaError`] values.
pub trait FormaErrorReqwestExt<T> {
    /// Maps a Reqwest error to a public kind and message.
    fn map_reqwest_forma_err<K: Into<FormaErrorKind>>(
        self,
        kind: K,
        message: &str,
    ) -> Result<T, FormaError>;
    /// Maps a Reqwest error and attaches optional structured details.
    fn map_reqwest_forma_err_with_detail<K: Into<FormaErrorKind>>(
        self,
        kind: K,
        message: &str,
        detail: Option<FormaErrorDetail>,
    ) -> Result<T, FormaError>;
}

impl<T> FormaErrorReqwestExt<T> for Result<T, reqwest::Error> {
    /// Maps a Reqwest error to a public kind and message.
    fn map_reqwest_forma_err<K: Into<FormaErrorKind>>(
        self,
        kind: K,
        message: &str,
    ) -> Result<T, FormaError> {
        self.map_reqwest_forma_err_with_detail(kind, message, None)
    }

    /// Maps a Reqwest error and attaches optional structured details.
    fn map_reqwest_forma_err_with_detail<K: Into<FormaErrorKind>>(
        self,
        kind: K,
        message: &str,
        detail: Option<FormaErrorDetail>,
    ) -> Result<T, FormaError> {
        self.map_err(|err| {
            let internal_info = expand_reqwest_error(err);
            FormaError {
                kind: kind.into(),
                internal_kind: Some(internal_info.internal_kind),
                message: message.to_string(),
                detail,
                non_saga_error: false,
                internal_info: Some(internal_info),
            }
        })
    }
}

/// Converts SurrealDB errors into [`FormaError`] values.
pub trait FormaErrorSurrealExt<T> {
    /// Maps a SurrealDB error to a public kind and message.
    fn map_surreal_forma_err<K: Into<FormaErrorKind>>(
        self,
        kind: K,
        message: &str,
    ) -> Result<T, FormaError>;
    /// Maps a SurrealDB error and attaches optional structured details.
    fn map_surreal_forma_err_with_detail<K: Into<FormaErrorKind>>(
        self,
        kind: K,
        message: &str,
        detail: Option<FormaErrorDetail>,
    ) -> Result<T, FormaError>;
}

impl<T> FormaErrorSurrealExt<T> for Result<T, surrealdb::Error> {
    /// Maps a SurrealDB error to a public kind and message.
    fn map_surreal_forma_err<K: Into<FormaErrorKind>>(
        self,
        kind: K,
        message: &str,
    ) -> Result<T, FormaError> {
        self.map_surreal_forma_err_with_detail(kind, message, None)
    }

    /// Maps a SurrealDB error and attaches optional structured details.
    fn map_surreal_forma_err_with_detail<K: Into<FormaErrorKind>>(
        self,
        kind: K,
        message: &str,
        detail: Option<FormaErrorDetail>,
    ) -> Result<T, FormaError> {
        self.map_err(|err| {
            let internal_info = expand_surrealdb_error(err);
            FormaError {
                kind: kind.into(),
                internal_kind: Some(internal_info.internal_kind),
                message: message.to_string(),
                detail,
                non_saga_error: false,
                internal_info: Some(internal_info),
            }
        })
    }
}

/// Classifies a MongoDB error and extracts its diagnostic information.
fn expand_mongodb_error(value: mongodb::error::Error) -> IntenalInfo {
    let mut kind: FormaInternalErrorKind = FormaInternalErrorKind::Unhandled;
    let mut message: Option<String> = None;
    let mut level: InternalLevel = InternalLevel::Error;

    match value.kind.as_ref() {
        mongodb::error::ErrorKind::InvalidArgument { message: msg, .. } => {
            kind = FormaErrorDatabase::InvalidArgument.into();
            message = Some(msg.clone());
        }
        mongodb::error::ErrorKind::Bson(err) => {
            message = Some(err.to_string());

            match err.kind {
                bson::error::ErrorKind::Binary { .. }
                | bson::error::ErrorKind::Decimal128 { .. }
                | bson::error::ErrorKind::DateTime { .. } => {
                    kind = FormaErrorApp::DataConflict.into();
                }

                bson::error::ErrorKind::ObjectId { .. } | bson::error::ErrorKind::Uuid { .. } => {
                    kind = FormaErrorDatabase::InvalidObjectId.into();
                }

                bson::error::ErrorKind::TooLargeUnsignedInteger { .. }
                | bson::error::ErrorKind::Utf8Encoding { .. } => {
                    kind = FormaErrorDatabase::InvalidArgument.into()
                }

                _ => kind = FormaErrorDatabase::IoStuck.into(),
            };
        }
        mongodb::error::ErrorKind::InsertMany(_) | mongodb::error::ErrorKind::BulkWrite(_) => {
            kind = FormaErrorDatabase::InsertError.into();
        }
        mongodb::error::ErrorKind::DnsResolve { message: msg, .. }
        | mongodb::error::ErrorKind::ServerSelection { message: msg, .. } => {
            kind = FormaErrorExternalService::ConnectionError.into();
            message = Some(msg.clone());
            level = InternalLevel::ClusterException;
        }
        mongodb::error::ErrorKind::Io(_)
        | mongodb::error::ErrorKind::InvalidResponse { .. }
        | mongodb::error::ErrorKind::Shutdown => {
            kind = FormaErrorDatabase::IoStuck.into();
            level = InternalLevel::Panic;
        }
        mongodb::error::ErrorKind::ConnectionPoolCleared { message: msg, .. } => {
            kind = FormaErrorExternalService::PoolError.into();
            message = Some(msg.clone());
        }
        mongodb::error::ErrorKind::IncompatibleServer { .. } => {
            kind = FormaErrorExternalService::Incompatible.into();
        }
        mongodb::error::ErrorKind::MissingResumeToken => {
            kind = FormaInternalErrorKind::Unhandled;
        }
        _ => {}
    };

    let message = format!("{} {}", value.to_string(), message.unwrap_or_default());

    IntenalInfo {
        target_kind: Some(value.kind.to_string()),
        internal_kind: kind,
        message: Some(message),
        stack_trace: value.source().map(|e| e.to_string()),
        level,
    }
}

/// Classifies a Redis error and extracts its diagnostic information.
fn expand_redis_error(value: redis::RedisError) -> IntenalInfo {
    let (kind, level) = {
        if value.is_timeout() {
            (FormaErrorCache::Timeout.into(), InternalLevel::Error)
        } else if value.is_io_error() {
            (
                FormaErrorCache::CacheUnavailable.into(),
                InternalLevel::Error,
            )
        } else if value.is_connection_dropped() || value.is_connection_refusal() {
            (
                FormaErrorExternalService::ConnectionError.into(),
                InternalLevel::ClusterException,
            )
        } else if value.is_cluster_error() {
            (
                FormaErrorCache::CacheUnavailable.into(),
                InternalLevel::ClusterException,
            )
        } else {
            match value.kind() {
                redis::ErrorKind::AuthenticationFailed | redis::ErrorKind::InvalidClientConfig => {
                    (FormaInternalErrorKind::Internal, InternalLevel::Panic)
                }
                _ => (FormaInternalErrorKind::Unhandled, InternalLevel::Error),
            }
        }
    };

    IntenalInfo {
        target_kind: Some(format!("{:?}", value.kind())),
        internal_kind: kind,
        message: Some(value.to_string()),
        stack_trace: value.source().map(|v| v.to_string()),
        level: level,
    }
}

/// Classifies a Reqwest error and extracts its diagnostic information.
fn expand_reqwest_error(error: reqwest::Error) -> IntenalInfo {
    IntenalInfo {
        target_kind: None,
        internal_kind: FormaErrorExternalService::ConnectionError.into(),
        message: Some(error.to_string()),
        stack_trace: error.source().map(|v| v.to_string()),
        level: InternalLevel::Error,
    }
}

/// Classifies a SurrealDB error and extracts its diagnostic information.
fn expand_surrealdb_error(error: surrealdb::Error) -> IntenalInfo {
    use surrealdb::types::ErrorDetails;

    let (internal_kind, level) = match error.details() {
        ErrorDetails::Validation(_) => (
            FormaErrorDatabase::InvalidArgument.into(),
            InternalLevel::Error,
        ),
        ErrorDetails::Query(_) => (
            FormaErrorDatabase::TransactionError.into(),
            InternalLevel::Error,
        ),
        ErrorDetails::Serialization(_) => (FormaErrorApp::InvalidType.into(), InternalLevel::Error),
        ErrorDetails::NotFound(_) => (FormaErrorDatabase::NotFound.into(), InternalLevel::Error),
        ErrorDetails::AlreadyExists(_) => (
            FormaErrorDatabase::DuplicateKey.into(),
            InternalLevel::Error,
        ),
        ErrorDetails::Connection(_) => (
            FormaErrorExternalService::ConnectionError.into(),
            InternalLevel::ClusterException,
        ),
        ErrorDetails::Thrown => (FormaInternalErrorKind::Unhandled, InternalLevel::Error),
        ErrorDetails::Configuration(_)
        | ErrorDetails::NotAllowed(_)
        | ErrorDetails::Internal
        | ErrorDetails::Context
        | &_ => (FormaInternalErrorKind::Internal, InternalLevel::Error),
    };

    IntenalInfo {
        target_kind: Some(error.kind_str().to_string()),
        internal_kind,
        message: Some(error.to_string()),
        stack_trace: error.source().map(|source| source.to_string()),
        level,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Identifies the internal category assigned to an underlying failure.
pub enum FormaInternalErrorKind {
    /// OAuth provider or protocol failure.
    OAuthError(FormaErrorOAuth),
    /// Database operation failure.
    DatabaseError(FormaErrorDatabase),
    /// Authentication failure.
    AuthError(FormaErrorAuth),
    /// Application or input failure.
    FormaError(FormaErrorApp),
    /// External-service failure.
    ExternalServiceError(FormaErrorExternalService),
    /// Cache failure.
    CacheError(FormaErrorCache),
    /// Unexpected internal failure.
    Internal,
    /// Failure without a more specific classification.
    #[default]
    Unhandled,
}

/// Converts a public error kind into its internal counterpart.
impl From<FormaErrorKind> for FormaInternalErrorKind {
    /// Preserves the category and variant in the internal representation.
    fn from(kind: FormaErrorKind) -> Self {
        match kind {
            FormaErrorKind::OAuthError(error) => Self::OAuthError(error),
            FormaErrorKind::DatabaseError(error) => Self::DatabaseError(error),
            FormaErrorKind::AuthError(error) => Self::AuthError(error),
            FormaErrorKind::FormaError(error) => Self::FormaError(error),
            FormaErrorKind::ExternalServiceError(error) => Self::ExternalServiceError(error),
            FormaErrorKind::CacheError(error) => Self::CacheError(error),
            FormaErrorKind::Internal => Self::Internal,
            FormaErrorKind::Unhandled => Self::Unhandled,
        }
    }
}

/// Converts an OAuth error into an internal error kind.
impl From<FormaErrorOAuth> for FormaInternalErrorKind {
    /// Wraps the OAuth variant as an internal kind.
    fn from(error: FormaErrorOAuth) -> Self {
        Self::OAuthError(error)
    }
}

/// Converts a database error into an internal error kind.
impl From<FormaErrorDatabase> for FormaInternalErrorKind {
    /// Wraps the database variant as an internal kind.
    fn from(error: FormaErrorDatabase) -> Self {
        Self::DatabaseError(error)
    }
}

/// Converts an authentication error into an internal error kind.
impl From<FormaErrorAuth> for FormaInternalErrorKind {
    /// Wraps the authentication variant as an internal kind.
    fn from(error: FormaErrorAuth) -> Self {
        Self::AuthError(error)
    }
}

/// Converts an application error into an internal error kind.
impl From<FormaErrorApp> for FormaInternalErrorKind {
    /// Wraps the application variant as an internal kind.
    fn from(error: FormaErrorApp) -> Self {
        Self::FormaError(error)
    }
}

/// Converts an external-service error into an internal error kind.
impl From<FormaErrorExternalService> for FormaInternalErrorKind {
    /// Wraps the external-service variant as an internal kind.
    fn from(error: FormaErrorExternalService) -> Self {
        Self::ExternalServiceError(error)
    }
}

/// Converts a cache error into an internal error kind.
impl From<FormaErrorCache> for FormaInternalErrorKind {
    /// Wraps the cache variant as an internal kind.
    fn from(error: FormaErrorCache) -> Self {
        Self::CacheError(error)
    }
}

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Classifies errors exposed by the application.
pub enum FormaErrorKind {
    #[error("OAuth error :: {0}")]
    /// OAuth provider or protocol failure.
    OAuthError(#[from] FormaErrorOAuth),
    #[error("database error :: {0}")]
    /// Database operation failure.
    DatabaseError(#[from] FormaErrorDatabase),
    #[error("auth error :: {0}")]
    /// Authentication failure.
    AuthError(#[from] FormaErrorAuth),
    #[error("app error :: {0}")]
    /// Application or input failure.
    FormaError(#[from] FormaErrorApp),
    #[error("external service error :: {0}")]
    /// External-service failure.
    ExternalServiceError(#[from] FormaErrorExternalService),
    #[error("cache error :: {0}")]
    /// Cache failure.
    CacheError(#[from] FormaErrorCache),
    #[error("internal")]
    /// Unexpected internal failure.
    Internal,
    #[default]
    #[error("unhandled")]
    /// Failure without a more specific classification.
    Unhandled,
}

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Authentication-specific error categories.
pub enum FormaErrorAuth {
    #[error("invalid signature")]
    /// Signature validation failed.
    InvalidSignature,
    #[error("token invalid")]
    /// Token is malformed or invalid.
    TokenInvalid,
    #[error("token revoked")]
    /// Token has been revoked.
    TokenRevoked,
    #[error("token expired")]
    /// Token has expired.
    TokenExpired,
    #[error("internal encryption fail")]
    /// An internal encryption operation failed.
    InternalEncryptionFail,
    #[error("invalid user agent")]
    /// User-agent data could not be accepted.
    InvalidUserAgent,
}

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// OAuth-specific error categories.
pub enum FormaErrorOAuth {
    #[error("unauthorized")]
    /// OAuth authorization was rejected.
    Unauthorized,
    #[error("not found")]
    /// The requested OAuth resource was not found.
    NotFound,
}

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Application-level error categories.
pub enum FormaErrorApp {
    #[error("option none")]
    /// A required optional value was absent.
    OptionNone,
    #[error("invalid type")]
    /// A value could not be parsed or had an unexpected type.
    InvalidType,
    #[error("invalid timestamp")]
    /// A timestamp could not be represented.
    InvalidTimestamp,
    #[error("invalid credentials")]
    /// Supplied credentials are invalid.
    InvalidCredentials,
    #[error("invalid version number")]
    /// A version value is unsupported or malformed.
    InvalidVersionNumber,
    #[error("data conflict")]
    /// Input data conflicts with an existing value or constraint.
    DataConflict,
}

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Database-specific error categories.
pub enum FormaErrorDatabase {
    #[error("invalid object id")]
    /// A database object identifier is malformed.
    InvalidObjectId,
    #[error("duplicate key")]
    /// A uniqueness constraint was violated.
    DuplicateKey,
    #[error("invalid argument")]
    /// A database operation received an invalid argument.
    InvalidArgument,
    #[error("io stuck")]
    /// A database I/O operation failed or stalled.
    IoStuck,
    #[error("insert error")]
    /// A database insert operation failed.
    InsertError,
    #[error("select error")]
    /// A database select operation failed.
    SelectError,
    #[error("not found")]
    /// The requested database record was not found.
    NotFound,
    #[error("update error")]
    /// A database update operation failed.
    UpdateError,
    #[error("delete error")]
    /// A database delete operation failed.
    DeleteError,
    #[error("transaction error")]
    /// A database transaction operation failed.
    TransactionError,
}

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Errors raised while calling an external service.
pub enum FormaErrorExternalService {
    #[error("URL error")]
    /// An external-service URL is invalid.
    UrlError,
    #[error("pool error")]
    /// A connection could not be acquired from a pool.
    PoolError,
    #[error("connection error")]
    /// A connection to an external service failed.
    ConnectionError,
    #[error("incompatible")]
    /// An external service is incompatible with the client.
    Incompatible,
}

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Cache-specific error categories.
pub enum FormaErrorCache {
    #[error("cache unavailable")]
    /// The cache service is unavailable.
    CacheUnavailable,
    #[error("time out")]
    /// A cache operation timed out.
    Timeout,
    #[error("io error")]
    /// A cache I/O operation failed.
    IOError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Confirms that the extension marks an error as unrelated to saga handling.
    fn marks_errors_as_non_saga() {
        let error = Err::<(), _>(FormaError::new(FormaErrorApp::DataConflict, "conflict"))
            .non_saga_error()
            .unwrap_err();

        assert!(error.non_saga_error);
    }

    #[test]
    /// Confirms that SurrealDB not-found errors map to the internal database kind.
    fn maps_surreal_not_found_to_internal_database_kind() {
        let result = Err::<(), _>(surrealdb::Error::not_found(
            "record not found".to_string(),
            None::<surrealdb::types::NotFoundError>,
        ))
        .map_surreal_forma_err(FormaErrorDatabase::SelectError, "failed to select");

        let error = result.unwrap_err();
        assert_eq!(
            error.internal_kind,
            Some(FormaInternalErrorKind::DatabaseError(
                FormaErrorDatabase::NotFound
            ))
        );
        assert_eq!(
            error.kind,
            FormaErrorKind::DatabaseError(FormaErrorDatabase::SelectError)
        );
    }
}
