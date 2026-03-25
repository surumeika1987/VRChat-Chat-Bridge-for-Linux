use std::sync::{ Arc, Mutex };
use anyhow::Result;
use tokio::{
    sync::mpsc
};
use crate::{
    osc::OSCCommand,
    ipc::IPCCommand,
};

slint::include_modules!();

/// UIウィンドウの生存期間を管理する型。
/// このインスタンスが保持されている間、GUIは表示されます。
/// ドロップされるとGUIは閉じます。
pub struct Ui {
    #[allow(dead_code)]
    ui: Arc<MainWindow>
}


impl Ui {
    const CLEAR_COUNT: u8 = 3;

    pub fn new(tx: mpsc::Sender<OSCCommand>, rx: mpsc::Receiver<IPCCommand>) -> Result<Self> {
        let ui = Arc::new(MainWindow::new()?);
        let input_text = Arc::new(Mutex::new(String::new()));
        let counter = Arc::new(Mutex::new(0));

        let cloned_input_text = Arc::clone(&input_text);
        ui.on_editing(move |text| {
            let text = text.to_string();
            let mut input_text = cloned_input_text.lock().unwrap();
            *input_text = text;
        });

        let weak = ui.as_weak();
        let cloned_counter = Arc::clone(&counter);
        let cloned_input_text = Arc::clone(&input_text);
        let cloned_tx = tx.clone();
        ui.on_submit(move |text| {
            let text = text.to_string();
            let cloned_tx = cloned_tx.clone();
            tokio::spawn(async move {
                cloned_tx.send(OSCCommand::SendChat { contents: text, immediately: true }).await.unwrap();
            });
            let mut counter = cloned_counter.lock().unwrap();
            *counter = 0;
            let mut input_text = cloned_input_text.lock().unwrap();
            *input_text = String::new();

            if let Some(ui) = weak.upgrade() {
                ui.set_input_text("".into());
            }
        });

        let cloned_ui = Arc::clone(&ui);
        Self::spawn_ui_ipc_command_bridge(cloned_ui, rx);

        let cloned_counter = Arc::clone(&counter);
        let cloned_input_text = Arc::clone(&input_text);
        Self::spawn_check_input_text(tx, cloned_input_text, cloned_counter);

        Ok(Ui { ui })
    }

    fn spawn_ui_ipc_command_bridge(
        ui: Arc<MainWindow>,
        mut rx: mpsc::Receiver<IPCCommand>,
    ) {
        let mut visible = false;
        let weak = ui.as_weak();
        tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                match cmd {
                    IPCCommand::Show => {
                        let _ = weak.upgrade_in_event_loop(|ui| {
                            let _ = ui.show();
                            ui.invoke_focus_input();
                        });
                        visible = true;
                    }
                    IPCCommand::Hide => {
                        let _ = weak.upgrade_in_event_loop(|ui| {
                            let _ = ui.hide();
                        });
                        visible = false;
                    }
                    IPCCommand::Toggle => {
                        let next = !visible;
                        let _ = weak.upgrade_in_event_loop(move |ui| {
                            if next {
                                let _ = ui.show();
                                ui.invoke_focus_input();
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

    fn spawn_check_input_text(
        tx: mpsc::Sender<OSCCommand>,
        input_text: Arc<Mutex<String>>,
        counter: Arc<Mutex<u8>>, 
    ) {
        tokio::spawn(async move {
            loop {
                let text = {
                    input_text.lock().unwrap().clone()
                };

                if text.trim().is_empty() {
                    let is_clear = {
                        let mut counter = counter.lock().unwrap();
                        if *counter == Self::CLEAR_COUNT {
                            *counter = 0;
                            return true;
                        }
                        *counter += 1;

                        false
                    };
                    if is_clear {
                        tx.send(
                                OSCCommand::SendChat{ 
                                    contents: "".to_string(),
                                    immediately: true,
                                }).await.unwrap();
                    }
                } else {
                    tx.send(OSCCommand::SendChat { contents: text, immediately: true }).await.unwrap();
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });
    }
}

