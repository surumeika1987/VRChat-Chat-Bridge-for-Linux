use std::sync::{Arc, Mutex};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufRead, AsyncWriteExt, AsyncBufReadExt,BufReader},
    net::{UnixListener, UnixStream},
    sync::mpsc,
};
use vrchat_osc::{
    rosc::{OscMessage, OscPacket, OscType},
    ServiceType, VRChatOSC,
};
use slint::ComponentHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum IPCCommand {
    Show,
    Hide,
    Toggle,
}

pub enum OSCCommand {
    SendChat { contents: String, immediately: bool },
    SetTyping { active: bool },
}

const IPC_SOCKET_PATH: &str = "/tmp/vrchat_chat_bridge.sock";

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

slint::include_modules!();

pub struct Ui{
    ui: MainWindow,
}

impl Ui {
    pub fn new(tx: mpsc::Sender<OSCCommand>, rx: mpsc::Receiver<IPCCommand>) -> Result<Self> {
        let ui = MainWindow::new()?;
        let input_text = Arc::new(Mutex::new(String::new()));

        Ui::spawn_ui_command_bridge(&ui, rx);

        let weak = ui.as_weak();
        let cloned_input_text = Arc::clone(&input_text);
        ui.on_editing(move |text| {
            let text = text.to_string();
            let mut input_text = cloned_input_text.lock().unwrap();
            *input_text = text;
        });

        let cloned_input_text = Arc::clone(&input_text);
        let cloned_tx = tx.clone();
        tokio::spawn(async move {
            loop {
                let text = {
                    cloned_input_text.lock().unwrap().clone()
                };

                if !text.trim().is_empty() {
                    cloned_tx.send(OSCCommand::SendChat { contents: text, immediately: true }).await;
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });

        let weak = ui.as_weak();
        let cloned_tx = tx.clone();
        ui.on_submit(move |text| {
            let text = text.to_string();
            let cloned_tx = cloned_tx.clone();
            tokio::spawn(async move {
                cloned_tx.send(OSCCommand::SendChat { contents: text, immediately: true }).await;
            });
            if let Some(ui) = weak.upgrade() {
                ui.set_input_text("".into());
            }
        });

        Ok(Ui { ui })
    }

    fn spawn_ui_command_bridge(
        ui: &MainWindow,
        mut rx: mpsc::Receiver<IPCCommand>,
    ) {
        let weak = ui.as_weak();
        let mut visible = false;

        tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                match cmd {
                    IPCCommand::Show => {
                        let _ = weak.upgrade_in_event_loop(|ui| {
                            let _ = ui.show();
                        });
                        visible = true;
                    }
                    IPCCommand::Hide => {
                        let _ = weak.upgrade_in_event_loop(|ui| {
                            let _ = ui.show();
                        });
                        visible = false;
                    }
                    IPCCommand::Toggle => {
                        let next = !visible;
                        let _ = weak.upgrade_in_event_loop(move |ui| {
                            if next {
                                let _ = ui.show();
                            } else {
                                let _ = ui.hide();
                            }
                        });
                        visible = next;
                    }
                }
            }
        });
    }
}

pub struct OSCManager;

impl OSCManager {
    pub async fn new(mut rx: mpsc::Receiver<OSCCommand>) -> Result<Self> {
        let vrchat_osc = VRChatOSC::new(None).await?;

        let cloned_vrchat_osc = vrchat_osc.clone();

        tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                match cmd {
                    OSCCommand::SendChat { contents, immediately } => {
                        cloned_vrchat_osc.send(
                            OscPacket::Message(OscMessage {
                                addr: "/chatbox/input".to_string(),
                                args: vec![
                                    OscType::String(contents),
                                    OscType::Bool(immediately),
                                ],
                            }),
                            "VRChat-Client-*",
                        ).await.unwrap();
                    }
                    OSCCommand::SetTyping { active } => {
                        cloned_vrchat_osc.send(
                            OscPacket::Message(OscMessage {
                                addr: "/chatbox/typing".to_string(),
                                args: vec![
                                    OscType::Bool(active),
                                ],
                            }),
                            "VRChat-Client-*",
                        ).await.unwrap();
                    }
                }
            }
        });

        Ok(OSCManager)
    }
}
