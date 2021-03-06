use crate::icon_data::get_icon_rgba;
use crate::miner_controller::MinerController;
use crate::miner_settings::*;

use eframe::{egui, epi};
use std::sync::Arc;
use std::sync::RwLock;
use tokio;
use tokio::sync::Mutex;

pub struct MinerError(String);

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct MinerApp {
    /// Stores the currently used settings
    pub settings: Arc<RwLock<MinerSettings>>,
    /// Stores the settings that haven't been applied yet
    temp_settings: MinerSettings,
    /// Reference to the MinerController
    pub miner_controller: Arc<Mutex<MinerController>>,
    /// Reference to the output of the miner process
    buffer: Arc<Mutex<Vec<String>>>,
    /// Reference to the repaint_signal, which is sent to when receiving
    /// updates from the controller
    pub repaint_signal: Option<Arc<dyn epi::backend::RepaintSignal>>,
    /// Used to contain the errors that are generated by the controller
    error: Arc<Mutex<Option<MinerError>>>,
}

impl MinerApp {
    pub async fn default() -> Self {
        let mc = MinerController::new();
        let buffer = mc.lock().await.buffer.clone();
        Self {
            settings: Arc::new(RwLock::new(MinerSettings::default())),
            temp_settings: MinerSettings::default(),
            miner_controller: mc.clone(),
            buffer,
            repaint_signal: None,
            error: Arc::new(Mutex::new(None)),
        }
    }

    /// Starts the listener for the update channel,
    /// requests repaint when receiving the update signal
    fn start_updater_task(&mut self, sender: tokio::sync::broadcast::Sender<()>) {
        let mut rcv = sender.subscribe();
        let repaint_signal = self.repaint_signal.clone();
        tokio::task::spawn(async move {
            loop {
                if rcv.recv().await.is_ok() {
                    if let Some(ref repaint) = repaint_signal {
                        repaint.request_repaint();
                    } else {
                        println!("Attempted repaint with None repaint_signal");
                    }
                }
            }
        });
    }

    /// Starts a listener on the controller error channel,
    /// listening to the miner_controller's error_tx.
    /// Mutates self.error when an error is received
    /// This only works for one error at a time
    pub async fn start_error_listener(&mut self) {
        let error_tx = self.miner_controller.lock().await.error_tx.clone();
        let mut rcv = error_tx.subscribe();
        let error = self.error.clone();
        tokio::task::spawn(async move {
            loop {
                if let Some(message) = rcv.recv().await.ok() {
                    *error.lock().await = Some(MinerError(message));
                }
            }
        });
    }

    fn show_device_settings(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Device Settings", |ui| {
            match self.temp_settings.device_type.as_mut() {
                None => {
                    if ui.button("Enable Device Settings").clicked() {
                        self.temp_settings.device_type = Some(DeviceType::Cuda(CudaSettings {
                            grid_size: "".to_string(),
                            block_size: "".to_string(),
                        }));
                    }
                }
                Some(x) => {
                    settings_entry("Device Type", ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.radio_value(
                                x,
                                DeviceType::Cuda(CudaSettings {
                                    grid_size: "".to_string(),
                                    block_size: "".to_string(),
                                }),
                                "Cuda",
                            );
                            ui.radio_value(
                                x,
                                DeviceType::OpenCl(ClSettings {
                                    global_work: "".to_string(),
                                    local_work: "".to_string(),
                                }),
                                "OpenCl",
                            );
                        });
                    });

                    match x {
                        DeviceType::Cuda(s) => {
                            settings_entry("Grid Size", ui, |ui| {
                                ui.add(egui::TextEdit::singleline(&mut s.grid_size));
                            });
                            settings_entry("Block Size", ui, |ui| {
                                ui.add(egui::TextEdit::singleline(&mut s.block_size));
                            });
                        }
                        DeviceType::OpenCl(s) => {
                            settings_entry("Global Work", ui, |ui| {
                                ui.add(egui::TextEdit::singleline(&mut s.global_work));
                            });
                            settings_entry("Local Work", ui, |ui| {
                                ui.add(egui::TextEdit::singleline(&mut s.local_work));
                            });
                        }
                    }

                    if ui.button("Disable Device Settings").clicked() {
                        self.temp_settings.device_type = None;
                    }
                }
            }
        });
    }

    fn show_ethminer_out(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .stick_to_bottom()
            .show(ui, |ui| {
                tokio::task::block_in_place(move || {
                    let b: &Vec<String> = &*self.buffer.blocking_lock();
                    b.into_iter().for_each(|line| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(line);
                        });
                    });
                });
            });
    }

    /// Shows recoverable errors with a closeable window
    /// This function will cause a panic if error is None when calling
    fn error_window(error: &mut Option<MinerError>, ctx: &egui::Context) {
        egui::Window::new("Error!")
            //.open(&mut true)
            .show(ctx, |ui| {
                ui.label(&error.as_ref().unwrap().0);
                if ui.button("Ok").clicked() {
                    *error = None;
                }
            });
    }
}

impl Drop for MinerApp {
    fn drop(&mut self) {
        MinerController::kill_child_miner(self.miner_controller.clone());
    }
}

impl epi::App for MinerApp {
    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::Context,
        frame: &epi::Frame,
        storage: Option<&dyn epi::Storage>,
    ) {
        // Attemps to load miner_settings from storage
        if let Some(s) = storage {
            if let Some(json) = s.get_string("miner_settings") {
                let msr: Result<MinerSettings, _> = serde_json::from_str(&json);
                match msr {
                    Ok(miner_settings) => {
                        println!("Parsed miner settings");
                        {
                            let mut settings = self.settings.write().unwrap();
                            *settings = miner_settings.clone();
                        }
                        self.temp_settings = miner_settings;
                    }
                    Err(e) => {
                        println!(
                            "could not parse miner settings from json: \"{json}\" error: \"{e}\""
                        );
                    }
                }
            } else {
                println!("miner_settings could not be retrieved from storage");
            }
        } else {
            println!("storage is None!");
        }

        // Assigns the repaint_signal
        tokio::task::block_in_place(|| {
            let rs: Arc<dyn epi::backend::RepaintSignal> =
                frame.0.lock().unwrap().repaint_signal.clone();
            self.repaint_signal = Some(rs);
            // Starts update_tx listener
            let update_tx = self.miner_controller.blocking_lock().updated_tx.clone();
            self.start_updater_task(update_tx);
        });
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &eframe::epi::Frame) {
        // Checks if this app has an error stored in self.error
        tokio::task::block_in_place(|| {
            let mut error = self.error.blocking_lock();

            if error.is_some() {
                MinerApp::error_window(&mut *error, ctx);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            for url in &mut self.temp_settings.url {
                settings_entry("Wallet Address", ui, |ui| {
                    ui.add(egui::TextEdit::singleline(&mut url.wallet_address));
                });
                settings_entry("Pool Address", ui, |ui| {
                    ui.add(egui::TextEdit::singleline(&mut url.pool));
                });
                settings_entry("Port", ui, |ui| {
                    ui.add(egui::TextEdit::singleline(&mut url.port));
                });

                ui.collapsing("Scheme", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Stratum");
                        ui.radio_value(&mut url.scheme.stratum, Stratum::stratum, "Stratum");
                        ui.radio_value(&mut url.scheme.stratum, Stratum::stratum1, "Stratum1");
                        ui.radio_value(&mut url.scheme.stratum, Stratum::stratum2, "Stratum2");
                        ui.radio_value(&mut url.scheme.stratum, Stratum::stratum3, "Stratum3");
                    });
                });
            }

            settings_entry("Ethminer Path", ui, |ui| {
                ui.add(egui::TextEdit::singleline(&mut self.temp_settings.bin_path));
                if ui.button("Choose Path").clicked() {
                    // Native file picker
                    let path = std::env::current_dir().unwrap();
                    let res = rfd::FileDialog::new().set_directory(&path).pick_files();

                    println!("Chose {:#?}", res);
                    match res {
                        Some(x) => {
                            if x.len() == 1 {
                                self.temp_settings.bin_path =
                                    x[0].to_str().expect("Could not get path").to_string();
                            }
                        }
                        None => {}
                    }
                }
            });

            self.show_device_settings(ui);

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    // Cancel temp_settings
                    self.temp_settings = self.settings.read().unwrap().clone();
                }
                if ui.button("Apply").clicked() {
                    {
                        let mut settings = self.settings.write().unwrap();
                        *settings = self.temp_settings.clone();
                    }
                    println!(
                        "Settings saved. New CLI options: {:?}",
                        &self.settings.read().unwrap().render()
                    );
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Run").clicked() {
                    MinerController::run_ethminer(
                        self.miner_controller.clone(),
                        self.settings.read().unwrap().clone(),
                    );
                }
                if ui.button("Stop").clicked() {
                    MinerController::kill_child_miner(self.miner_controller.clone());
                }
            });

            ui.vertical_centered_justified(|ui| {
                self.show_ethminer_out(ui);
            });
        });
    }

    fn name(&self) -> &str {
        "etherminer-gui"
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        // Saves some settings
        let miner_settings = serde_json::to_string(&self.settings);
        match miner_settings {
            Ok(json) => {
                storage.set_string("miner_settings", json.clone());
                println!("Saved miner_settings \"{json}\"");
            }
            _ => {}
        }
    }
}

fn settings_entry<R>(
    label: &'static str,
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<()> {
    ui.horizontal(|ui| {
        ui.centered_and_justified(|ui| {
            ui.label(label);
            add_contents(ui);
        });
    })
}
