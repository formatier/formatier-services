//! General-purpose helpers shared by Formatier services.
use crate::domain::entities::{FormaError, FormaErrorApp, FormaErrorConverter};

/// Declares modules and re-exports their contents from the current module.
#[macro_export]
macro_rules! inline_mod {
    ($($name:ident), + $(,)?) => {
        $(
            mod $name; pub use $name::*;
        )+
    };
}

#[macro_export]
/// Declares a string constant selected by the build profile.
/// Debug builds use the development value; optimized builds use the production value.
macro_rules! use_env {
    ($name:ident, $dev_env:expr, $prod_env:expr) => {
        #[cfg(not(debug_assertions))]
        pub const $name: &str = $prod_env;

        #[cfg(debug_assertions)]
        pub const $name: &str = $dev_env;
    };
}

/// Parses a string-like value as a UUID.
pub fn string_to_uuid(s: impl Into<String>) -> Result<uuid::Uuid, FormaError> {
    uuid::Uuid::parse_str(s.into().as_str())
        .map_forma_err(FormaErrorApp::InvalidType, "cannot parse string as UUID")
}

/// Installs the default Rustls cryptographic provider.
pub fn init_rustls() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install the Rustls cryptographic provider");
}
