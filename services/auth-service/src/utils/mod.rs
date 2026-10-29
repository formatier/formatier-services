//! Authentication service-specific utility functions.

use std::str::FromStr;

use forma_core::domain::entities::{FormaError, FormaErrorConverter, FormaErrorDatabase};
use surrealdb::types;

pub fn parse_uuid(things: &str) -> Result<types::Uuid, FormaError> {
    types::Uuid::from_str(things)
        .map_forma_err(FormaErrorDatabase::InvalidObjectId, "failed to parse something to uuid")
}

mod test {
    use super::*;

    #[test]
    fn record_id_helpers_accept_uuid_values() {
        let id = types::Uuid::new_v4();
        assert_eq!(parse_uuid(&id.to_string()).unwrap(), id);
    }

    #[test]
    fn record_id_helpers_reject_malformed_values() {
        assert!(parse_uuid("not-a-uuid").is_err());
    }
}