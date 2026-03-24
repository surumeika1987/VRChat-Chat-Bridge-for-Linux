use serde::{Deserialize, Serialize};

pub mod server;
pub mod client;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum IPCCommand {
    Show,
    Hide,
    Toggle,
}

const IPC_SOCKET_PATH: &str = "/tmp/vrchat_chat_bridge.sock";
