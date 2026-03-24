use anyhow::Result;
use tokio::{
    sync::mpsc,
};
use vrchat_osc::{
    rosc::{ OscMessage, OscPacket, OscType, },
    VRChatOSC, 
};

pub enum OSCCommand {
    SendChat { contents: String, immediately: bool },
    SetTyping { active: bool },
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
