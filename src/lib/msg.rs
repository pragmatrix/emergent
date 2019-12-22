use crate::test_runner::TestEnvironment;
use crate::test_watcher;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Msg {
    #[serde(skip)]
    WatcherNotification(Result<test_watcher::Notification, failure::Error>),
    ToggleTestcase {
        name: String,
    },
    #[serde(skip)]
    RerunTestcases(TestEnvironment),
}
