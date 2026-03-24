use std::sync::{ Arc, Mutex };
use anyhow::Result;
use tokio::{
    sync::mpsc,
};
use crate::{
    osc::OSCCommand,
    ipc::IPCCommand,
};

slint::include_modules!();

pub struct Ui{
    ui: MainWindow,
}


impl Ui {
    pub fn new(tx: mpsc::Sender<OSCCommand>, rx: mpsc::Receiver<IPCCommand>) -> Result<Self> {
        let ui = MainWindow::new()?;
        let input_text = Arc::new(Mutex::new(String::new()));
        const CLEAR_COUNT: u8 = 3;
        let counter = Arc::new(Mutex::new(0));

        Ui::spawn_ui_command_bridge(&ui, rx);

        let cloned_input_text = Arc::clone(&input_text);
        ui.on_editing(move |text| {
            let text = text.to_string();
            let mut input_text = cloned_input_text.lock().unwrap();
            *input_text = text;
        });

        let cloned_input_text = Arc::clone(&input_text);
        let cloned_counter = Arc::clone(&counter);
        let cloned_tx = tx.clone();
        tokio::spawn(async move {
            loop {
                let text = {
                    cloned_input_text.lock().unwrap().clone()
                };

                if text.trim().is_empty() {
                    let mut is_clear = false;
                    {
                        let mut counter = cloned_counter.lock().unwrap();
                        if *counter == CLEAR_COUNT {
                            *counter = 0;
                            is_clear = true;
                        }
                        *counter += 1;
                    }
                    if is_clear {
                        cloned_tx
                            .send(
                                OSCCommand::SendChat{ 
                                    contents: "".to_string(),
                                    immediately: true,
                                }).await;
                    }
                } else {
                    cloned_tx.send(OSCCommand::SendChat { contents: text, immediately: true }).await;
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });

        let weak = ui.as_weak();
        let cloned_counter = Arc::clone(&counter);
        let cloned_input_text = Arc::clone(&input_text);
        let cloned_tx = tx.clone();
        ui.on_submit(move |text| {
            let text = text.to_string();
            let cloned_tx = cloned_tx.clone();
            tokio::spawn(async move {
                cloned_tx.send(OSCCommand::SendChat { contents: text, immediately: true }).await;
            });
            let mut counter = cloned_counter.lock().unwrap();
            *counter = 0;
            let mut input_text = cloned_input_text.lock().unwrap();
            *input_text = String::new();

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

