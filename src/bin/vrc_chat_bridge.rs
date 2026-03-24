use vrchat_chat_bridge::{
    ipc::{
        IPCCommand,
        client::IPCClient,
        server::IPCServer,
    },
    osc::OSCManager,
    ui::Ui,
};
use anyhow::Result;
use tokio::sync::mpsc;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if 1 < args.len() {
        if args[1] == "toggle" {
            IPCClient::ipc_send(&IPCCommand::Toggle).await?;
            return Ok(());
        } else if args[1] == "show" {
            IPCClient::ipc_send(&IPCCommand::Show).await?;
            return Ok(());
        } else if args[1] == "hide" {
            IPCClient::ipc_send(&IPCCommand::Hide).await?;
            return Ok(());
        }
    }

    let (ipc_tx, ipc_rx) = mpsc::channel(32);
    let (osc_tx, osc_rx) = mpsc::channel(256);

    IPCServer::new(ipc_tx)?;

    OSCManager::new(osc_rx).await?;
    
    let _ui = Ui::new(osc_tx, ipc_rx)?;

    slint::run_event_loop_until_quit()?;

    Ok(())
}
