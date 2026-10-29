//! Domain entities and token/account types used by the authentication service.
use std::{collections::HashMap, fmt::Display};

use forma_core::{
    domain::entities::{
        BsonTime, ChronoTime, Model, ModelPairs, SurrealTime, Table, TablePairs, Timestamp,
        UpdaterFilter, UpdaterMode,
    },
    inline_mod,
};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use surrealdb::types::{RecordId, SurrealValue, Value};

inline_mod!(oauth);
inline_mod!(dto);

#[derive(EnumString, PartialEq, Display)]
#[strum(serialize_all = "snake_case")]
/// Classifies authentication token purposes.
pub enum TokenType {
    /// Token authorizing API access.
    Access,
    /// Token used to refresh an access session.
    Refresh,
    /// Token used to initialize an account.
    Initialization,
}

#[derive(Serialize, Deserialize, Default, SurrealValue, Clone)]
/// Stores an account's core profile and linked OAuth identities.
pub struct Account {
    /// Primary email address for the account.
    pub email: String,
    /// Email verification status.
    pub email_verified: bool,
    /// Optional account display name.
    pub user_name: Option<String>,
    /// Optional password credential.
    #[serde(skip)]
    pub password: Option<String>,
}

pub struct AccountUpdaterFilter;
impl UpdaterFilter<&'static str> for AccountUpdaterFilter {
    const FILTER: UpdaterMode<&str> = UpdaterMode::Include(&["user_name"]);
}

#[derive(Serialize, Deserialize, SurrealValue, Copy, Clone)]
#[serde(rename_all = "lowercase")]
#[surreal(rename_all = "lowercase")]
/// Identifies an external OAuth provider.
pub enum ProviderIdent {
    /// Google OAuth provider.
    Google,
    /// GitHub OAuth provider.
    Github,
}

#[derive(Serialize, Deserialize, SurrealValue, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
#[surreal(tag = "type", rename_all = "lowercase")]
/// Stores the provider-specific identifier linked to an account.
pub struct Provider {
    /// Email verification status.
    pub email_verified: bool,
    /// Provider email that user has.
    pub email: String,
}

#[derive(Serialize, Deserialize, SurrealValue, Clone)]
/// Stores a user's public profile and badges.
pub struct User {
    /// Name displayed for the user.
    pub display_name: String,
    /// Optional given name.
    pub name: Option<String>,
    /// Optional middle name.
    pub middle_name: Option<String>,
    /// Optional family name.
    pub family_name: Option<String>,
    /// Optional user description.
    pub description: Option<String>,
    /// Optional identifier of the profile image bucket.
    pub profile_bucket_id: Option<String>,
    /// Indicates whether the profile is verified by Formatier.
    pub formatier_verified: bool,
}

pub struct UserUpdaterFilter;
impl UpdaterFilter<&'static str> for UserUpdaterFilter {
    const FILTER: UpdaterMode<&str> =
        UpdaterMode::Exclude(&["formatier_verified", "profile_bucket_id"]);
}

#[derive(Serialize, Deserialize, SurrealValue, Clone)]
/// Describes a badge awarded or assigned to a user.
pub struct Badge {
    /// Badge name.
    pub name: String,
    /// Optional explanatory text.
    pub description: Option<String>,
    /// Category used to classify the badge.
    pub category: Category,
}

#[derive(Serialize, Deserialize, SurrealValue, Clone)]
/// Classifies a user badge.
pub enum Category {
    /// Badge awarded for a general achievement.
    Achievement,
    /// Badge associated with an event.
    Event,
    /// Badge associated with a challenge.
    Challenge,
    /// Badge associated with a competition.
    Competition,
    /// Badge representing a reward.
    Reward,
    /// Badge confirming Formatier verification.
    FormatierVerified,
}

#[derive(Serialize, Deserialize)]
/// Claims carried by an account initialization token.
pub struct InitializationTokenClaims {
    #[serde(rename = "sub")]
    /// Account identifier associated with the initialization token.
    pub account_id: String,
}

#[derive(Serialize, Deserialize)]
/// Claims carried by an access token.
pub struct AccessTokenClaims {
    #[serde(rename = "jti")]
    /// Session identifier associated with the access token.
    pub session_id: String,
    #[serde(rename = "sub")]
    /// Account identifier associated with the access token.
    pub account_id: String,

    /// Email address associated with the account.
    pub email: String,
    /// User identifier associated with the account.
    pub user_id: String,

    /// Permissions granted by the token.
    pub scope: Box<[String]>,
}

#[derive(Serialize, Deserialize)]
/// Claims carried by a refresh token.
pub struct RefreshTokenClaims {
    #[serde(rename = "jti")]
    /// Session identifier to refresh.
    pub session_id: String,
}

#[derive(Serialize, Deserialize)]
/// Standard JWT fields combined with application-specific claims.
pub struct TokenClaims<T> {
    #[serde(rename = "iss")]
    /// Identity of the token issuer.
    pub issuer: Issuer,
    #[serde(rename = "aud")]
    /// Services allowed to accept the token.
    pub audience: Box<[Audience]>,

    #[serde(rename = "exp")]
    /// Token expiration timestamp.
    pub expiration_time: Timestamp<ChronoTime>,
    #[serde(rename = "iat")]
    /// Token issuance timestamp.
    pub issue_at: Timestamp<ChronoTime>,

    /// Application-specific token claims.
    pub claims: T,
}

#[derive(Serialize, Deserialize)]
/// Identifies a service that may accept a token.
pub enum Audience {
    #[serde(rename = "formatier-api")]
    /// Formatier API.
    FormatierAPI,
    #[serde(rename = "formatier-mail-server")]
    /// Formatier mail server.
    FormatierMailServer,
    #[serde(rename = "formatine-server")]
    /// Formatine server.
    FormatineServer,
    #[serde(rename = "formatier-bucket-server")]
    /// Formatier bucket server.
    FormatierBucketServer,
}

#[derive(Serialize, Deserialize)]
/// Identifies the authority that issued a token.
pub enum Issuer {
    #[serde(rename = "formatier")]
    /// Formatier token issuer.
    Formatier,
}

#[derive(Serialize, Deserialize, SurrealValue, Clone)]
/// Captures user-agent attributes associated with a session.
pub struct Session {
    /// Operating system name.
    pub os: String,
    /// Operating system version.
    pub os_version: String,
    /// Browser name.
    pub browser: String,
    /// Browser version.
    pub browser_version: String,
    /// Device or browser vendor.
    pub vendor: String,
}

/// Converts parsed Woothee user-agent data into a session record.
impl From<woothee::parser::WootheeResult<'_>> for Session {
    /// Maps the parser's fields to the session representation.
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

#[derive(Serialize, Deserialize, Default, SurrealValue, Clone)]
/// Stores account timestamps and its optional associated user identifier.
pub struct AccountMetadata {
    /// Account creation timestamp.
    pub create_at: Timestamp<SurrealTime>,
    /// Timestamp of the most recent account update.
    pub update_at: Timestamp<SurrealTime>,

    /// Identifier of the user initialized for this account, if any.
    pub user: Option<RecordId>,
}

#[derive(Serialize, Deserialize, Default, SurrealValue, Clone)]
/// Stores creation and update timestamps for a user.
pub struct UserMetadata {
    /// User creation timestamp.
    pub create_at: Timestamp<SurrealTime>,
    /// Timestamp of the most recent user update.
    pub update_at: Timestamp<SurrealTime>,
}

#[derive(Serialize, Deserialize, SurrealValue, Clone)]
/// Stores session timestamps and the owning account identifier.
pub struct SessionMetadata {
    /// Session creation timestamp.
    pub create_at: Timestamp<SurrealTime>,
    /// Timestamp of the most recent session update.
    pub update_at: Timestamp<SurrealTime>,

    /// Identifier of the account that owns the session.
    pub account: RecordId,
}

#[derive(Serialize, Deserialize, SurrealValue, Clone)]
/// Stores session timestamps and the owning account identifier.
pub struct ProviderMetadata {
    /// Session creation timestamp.
    pub create_at: Timestamp<SurrealTime>,
    /// Timestamp of the most recent session update.
    pub update_at: Timestamp<SurrealTime>,

    /// Identifier of the account that owns the session.
    pub account: RecordId,
}

/// Account table parameterized by its persistence state.
pub type AccountTable<S> = Table<Account, AccountMetadata, S>;
/// User table parameterized by its persistence state.
pub type UserTable<S> = Table<User, UserMetadata, S>;
/// Session table parameterized by its persistence state.
pub type SessionTable<S> = Table<Session, SessionMetadata, S>;
/// Provider table parameterized by its persistence state.
pub type ProviderTable<S> = Table<Provider, ProviderMetadata, S>;
// Badge table parameterized by its persistence state.
//pub type BadgeTable<S> = Table<Badge, BadgeMetadata, S>;

/// Data and metadata used to construct an account table.
pub type AccountTablePairs = TablePairs<Account, AccountMetadata>;
/// Data and metadata used to construct a user table.
pub type UserTablePairs = TablePairs<User, UserMetadata>;
/// Data and metadata used to construct a session table.
pub type SessionTablePairs = TablePairs<Session, SessionMetadata>;
/// Data and metadata used to construct a provider table.
pub type ProviderTablePairs = TablePairs<Provider, ProviderMetadata>;
// Data and metadata used to construct a badge table.
//pub type BadgeTablePairs = TablePairs<Badge, BadgeMetadata>;
