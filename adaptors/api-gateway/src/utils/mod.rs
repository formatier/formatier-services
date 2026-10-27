//! Helpers for loading API gateway configuration.
use std::{fs, io::Read};

use forma_core::domain::entities::{FormaError, FormaErrorConverter, FormaErrorKind};

use crate::domain::entities::Config;

/// Loads and parses the gateway configuration from `./route.yml`.
pub fn load_config() -> Result<Config, FormaError> {
    let mut file = fs::File::open("./route.yml")
        .map_forma_err(FormaErrorKind::Internal, "Could not open the route config")?;
    let mut raw_yml = String::new();

    file.read_to_string(&mut raw_yml)
        .map_forma_err(FormaErrorKind::Internal, "Could not read the config")?;

    let config: Config = serde_yaml_ng::from_str(&raw_yml)
        .map_forma_err(FormaErrorKind::Internal, "Could not parse the config")?;

    Ok(config)
}
