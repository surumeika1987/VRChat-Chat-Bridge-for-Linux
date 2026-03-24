use anyhow::Result;
use tokio::{
    net::UnixStream,
    io::AsyncWriteExt,
};
use super::{IPC_SOCKET_PATH, IPCCommand};

pub struct IPCClient;

impl IPCClient {
    pub async fn ipc_send(cmd: &IPCCommand) -> Result<()> {
        let mut stream = UnixStream::connect(IPC_SOCKET_PATH).await?;
        let json = serde_json::to_string(cmd)?;
        stream.write_all(json.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        Ok(())
    }
}
