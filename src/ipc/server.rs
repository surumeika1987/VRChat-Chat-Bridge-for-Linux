use anyhow::Result;
use tokio::{
    net::{UnixStream, UnixListener},
    io::{BufReader, AsyncBufReadExt},
    sync::mpsc,
};
use super::{IPC_SOCKET_PATH, IPCCommand};

pub struct IPCServer;

impl IPCServer {
    pub fn new(tx: mpsc::Sender<IPCCommand>) -> Result<Self> {
        let _ = std::fs::remove_file(IPC_SOCKET_PATH);
        let listener = UnixListener::bind(IPC_SOCKET_PATH)?;

        tokio::spawn(async move {
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let tx = tx.clone();

                tokio::spawn(async move {
                    if let Err(e) = IPCServer::handle_ipc_clinet(stream, tx).await {
                        eprint!("IPC client error: {e}");
                    }
                });
            }
        });

        Ok(IPCServer)
    }

    async fn handle_ipc_clinet(
        stream: UnixStream,
        tx: mpsc::Sender<IPCCommand>,
    ) -> Result<()> {
        let reader = BufReader::new(stream);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            let cmd: IPCCommand = serde_json::from_str(&line)?;
            tx.send(cmd).await?;
        }

        Ok(())
    }
}
