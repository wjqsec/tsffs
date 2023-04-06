//! Messages generated by the module

use serde::{Deserialize, Serialize};

use crate::{
    module::{config::OutputConfig, stop_reason::StopReason},
    state::ConfuseModuleInput,
};

use super::Message;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModuleMessage {
    /// Mesage
    Initialized(OutputConfig),
    /// Message indicating ready to run
    Ready,
    /// Message indicating stopped and the reason why, for example a crash, normal run finished,
    /// or some error occurred.
    Stopped(StopReason),
}
