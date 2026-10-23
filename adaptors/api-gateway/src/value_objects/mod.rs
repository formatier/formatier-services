//! Identifiers used to select configured API gateway upstreams.
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "snake_case")]
/// Identifies an upstream service in gateway configuration.
pub enum UpstreamKey {
    MonitorServer,
    FormatineOperator,
    FormatineWorker,
    MailServer,
    IntegrationsGateway,

    AuthService,
    ForumService,
    InterspaceService,
    KeyStoneService,
    NotificationService,
    PaymentService,
}
