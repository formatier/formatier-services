use std::{fs, io::Read};

use forma_core::domain::entities::{
    FormaError, FormaErrorConverter, FormaErrorKind,
};

use crate::domain::entities::Config;

pub fn load_config() -> Result<Config, FormaError> {
    let mut file = fs::File::open("./deployment/config/route.yml")
        .map_forma_err(FormaErrorKind::Internal, "cannot open route config")?;
    let mut raw_yml = String::new();

    file.read_to_string(&mut raw_yml)
        .map_forma_err(FormaErrorKind::Internal, "cannot read config")?;

    let config: Config = serde_yaml_ng::from_str(&raw_yml)
        .map_forma_err(FormaErrorKind::Internal, "cannot parse config")?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_real_config() {
        let _ = load_config().unwrap();
    }
}
