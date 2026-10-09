use std::collections::HashMap;

use forma_core::domain::entities::{BsonTime, ChronoTime, Model, ModelPairs, Timestamp};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::inline_mod;

inline_mod!(dto);

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ProviderV1 {
    Google,
    Github,
}

#[derive(Serialize, Deserialize, Default)]
pub struct AccountV1 {
    pub email: String,
    pub user_name: Option<String>,
    pub password: Option<String>,

    pub providers: HashMap<ProviderV1, AccountV1Provider>,
}

#[derive(Serialize, Deserialize)]
pub struct AccountV1Provider {
    pub provider_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserV1 {
    pub display_name: String,
    pub name: Option<String>,
    pub middle_name: Option<String>,
    pub family_name: Option<String>,
    pub description: Option<String>,
    pub profile_bucket_id: Option<String>,
    pub formatier_verified: bool,
}

#[derive(Serialize, Deserialize)]
pub struct BadgeV1 {
    pub name: String,
    pub description: Option<String>,
    pub category: CategoryV1,
}

#[derive(Serialize, Deserialize)]
pub enum CategoryV1 {
    Achievement,
    Event,
    Challenge,
    Competition,
    Reward,
    FormatierVerified,
}

#[derive(Serialize, Deserialize)]
pub struct InitializationTokenClaimsV1 {
    #[serde(rename = "sub")]
    pub account_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct AccessTokenClaimsV1 {
    #[serde(rename = "jti")]
    pub session_id: String,
    #[serde(rename = "sub")]
    pub account_id: String,

    pub email: String,
    pub user_id: String,

    pub scope: Box<[String]>,
}

#[derive(Serialize, Deserialize)]
pub struct RefreshTokenClaimsV1 {
    #[serde(rename = "jti")]
    pub session_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct TokenClaimsV1<T> {
    #[serde(rename = "iss")]
    pub issuer: IssuerV1,
    #[serde(rename = "aud")]
    pub audience: Box<[AudienceV1]>,

    #[serde(rename = "exp")]
    pub expiration_time: Timestamp<ChronoTime>,
    #[serde(rename = "iat")]
    pub issue_at: Timestamp<ChronoTime>,

    pub claims: T,
}

#[derive(Serialize, Deserialize)]
pub enum AudienceV1 {
    #[serde(rename = "formatier-api")]
    FormatierAPI,
    #[serde(rename = "formatier-mail-server")]
    FormatierMailServer,
    #[serde(rename = "formatier-orchestration-server")]
    FormatierOrchestrationServer,
    #[serde(rename = "formatier-bucket-server")]
    FormatierBucketServer,
}

#[derive(Serialize, Deserialize)]
pub enum IssuerV1 {
    #[serde(rename = "formatier")]
    Formatier,
}

#[derive(Serialize, Deserialize)]
pub struct SessionV1 {
    pub os: String,
    pub os_version: String,
    pub browser: String,
    pub browser_version: String,
    pub vendor: String,
}

impl From<woothee::parser::WootheeResult<'_>> for SessionV1 {
    fn from(value: woothee::parser::WootheeResult) -> Self {
        Self {
            os: value.os.into(),
            os_version: value.os_version.into(),
            browser: value.browser_type.into(),
            browser_version: value.version.into(),
            vendor: value.vendor.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct AccountV1Metadata {
    pub create_at: Timestamp<BsonTime>,
    pub update_at: Timestamp<BsonTime>,

    pub user_id: Option<ObjectId>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct UserV1Metadata {
    pub create_at: Timestamp<BsonTime>,
    pub update_at: Timestamp<BsonTime>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct SessionV1Metadata {
    pub create_at: Timestamp<BsonTime>,
    pub update_at: Timestamp<BsonTime>,

    pub account_id: ObjectId,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "version", bound = "")]
pub enum AccountModel<S> {
    #[serde(rename = "v1")]
    V1(Model<AccountV1, AccountV1Metadata, S>),
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "version", bound = "")]
pub enum UserModel<S> {
    #[serde(rename = "v1")]
    V1(Model<UserV1, UserV1Metadata, S>),
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "version", bound = "")]
pub enum SessionModel<S> {
    #[serde(rename = "v1")]
    V1(Model<SessionV1, SessionV1Metadata, S>),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum AccountModelPairs {
    #[serde(rename = "v1")]
    V1(ModelPairs<AccountV1, AccountV1Metadata>),
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum UserModelPairs {
    #[serde(rename = "v1")]
    V1(ModelPairs<UserV1, UserV1Metadata>),
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum SessionModelPairs {
    #[serde(rename = "v1")]
    V1(ModelPairs<SessionV1, SessionV1Metadata>),
}
