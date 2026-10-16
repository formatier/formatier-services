use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "snake_case")]
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
