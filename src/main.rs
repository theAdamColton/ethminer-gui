// This makes the compiled windows app not launch with a console window
#![windows_subsystem = "windows"]

mod icon_data;
mod miner_controller;
mod miner_settings;
/**
    Adam Colton 2021

 Simple GUI app for ethminer

  _________________________________________
 |file|options|   etherminer-gui     |-|/|X|
 |-----------------------------------------|
 | Settings:    _______________________    |
 ||wallet ad|  |_->_cuda__|_->_opencl__|   |    radio button
 ||pool adre|  | ....collapseable....  |   |    shows either cuda or opencl specific settings
 ||stratum  |  |__________|____________|   |
 ||transport|                              |
 |                                         |
 |  |cancel|         |apply and restart|   |
 |-----------------------------------------|
 |              ________________________   |
 |  |rr   |    |                        |  |
 |  |stop |    |     hr /time graph     |  |
 |             |________________________|  |
 |              ________________________   |
 |  |start|    |                        |  |
 |  |stop |    |     etherminer sout    |  |    expandable upper panel (window with panels)
 |  |pause|    |________________________|  |
 |_________________________________________|

  _options_________
 | set bin address |
 |launch on startup|
 | ....            |
 |_________________|

  _file____________
 | save profile    |
 | load profile    |
 | set default prof|
 |_________________|

*/
extern crate strum;
#[macro_use]
extern crate strum_macros;

use icon_data::get_icon_data;
use miner_controller::MinerController;

use eframe::{egui, epi};
use miner_settings::*;

use std::sync::Arc;
use tokio;
use tokio::sync::Mutex;

pub struct MinerError(&'static str);

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct MinerApp {
    /// Stores the currently used settings
    settings: Arc<MinerSettings>,
    /// Stores the settings that haven't been applied yet
    temp_settings: MinerSettings,
    /// Reference to the MinerController
    #[cfg_attr(feature = "persistence", serde(skip))]
    miner_controller: Arc<Mutex<MinerController>>,
    /// Reference to the output of the miner process
    buffer: Arc<Mutex<Vec<String>>>,
    /// Reference to the repaint_signal, which is sent to when receiving
    /// updates from the controller
    #[cfg_attr(feature = "persistence", serde(skip))]
    repaint_signal: Option<Arc<dyn epi::backend::RepaintSignal>>,
    /// Used to contain the errors that are generated by the controller
    #[cfg_attr(feature = "persistence", serde(skip))]
    error: Arc<Mutex<Option<MinerError>>>,
}

impl MinerApp {
    /// Aquires the lock and sends to the spawn channel
    /// Sends a reference to the MinerSettings to the controller
    fn run_ethminer(&self) {
        let mc = self.miner_controller.clone();
        let settings = self.settings.clone();
        tokio::spawn(async move {
            mc.lock()
                .await
                .spawn_tx
                .send(settings)
                .await
                .expect("Could not send spawn");
        });
    }

    /// Aquires the lock and sends to the kill channel
    fn kill_child_miner(&self) {
        let mc = self.miner_controller.clone();
        tokio::spawn(async move {
            println!("spawned");
            mc.lock()
                .await
                .kill_tx
                .send(())
                .await
                .expect("Could not send kill");
        });
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

    /// Starts a listener on the controller error channel
    /// Mutates self.error when an error is received
    /// This only works for one error at a time
    fn start_error_listener(&mut self, sender: tokio::sync::broadcast::Sender<&'static str>) {
        let mut rcv = sender.subscribe();
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
                ui.label(error.as_ref().unwrap().0);
                if ui.button("Ok").clicked() {
                    *error = None;
                }
            });
    }
}

impl Drop for MinerApp {
    fn drop(&mut self) {
        self.kill_child_miner();
    }
}

impl epi::App for MinerApp {
    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::Context,
        frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Assigns the repaint_signal if it is None
        tokio::task::block_in_place(|| {
            let rs: Arc<dyn epi::backend::RepaintSignal> =
                frame.0.lock().unwrap().repaint_signal.clone();
            self.repaint_signal = Some(rs);
            println!(
                "Set repaint_signal, repaint_signal is None? {}",
                self.repaint_signal.is_none()
            );
            // Starts update_tx listener
            let update_tx = self.miner_controller.blocking_lock().updated_tx.clone();
            self.start_updater_task(update_tx);
        });

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &eframe::epi::Frame) {
        println!("repaint_signal is None? {}", self.repaint_signal.is_none());
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
                    self.temp_settings = (*self.settings).clone();
                }
                if ui.button("Apply").clicked() {
                    self.settings = Arc::new(self.temp_settings.clone());
                    println!("{:?}", &self.settings.render());
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Run").clicked() {
                    self.run_ethminer();
                }
                if ui.button("Stop").clicked() {
                    self.kill_child_miner();
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
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }
}

#[tokio::main]
async fn main() {
    let mc = MinerController::new();
    let buffer = mc.lock().await.buffer.clone();
    let mut app: MinerApp = MinerApp {
        settings: Arc::new(MinerSettings::default()),
        temp_settings: MinerSettings::default(),
        miner_controller: mc.clone(),
        buffer,
        repaint_signal: None,
        error: Arc::new(Mutex::new(None)),
    };

    // Starts some listeners

    let error_tx = mc.lock().await.error_tx.clone();
    app.start_error_listener(error_tx);

    // Gets the icon
    let icon: Vec<u8> = get_icon_data();
    let icon_data = epi::IconData {
        rgba: icon,
        width: 64,
        height: 64,
    };

    let native_options = eframe::NativeOptions {
        resizable: false,
        drag_and_drop_support: false,
        min_window_size: Some(egui::Vec2 { x: 500.0, y: 300.0 }),
        initial_window_size: Some(egui::Vec2 { x: 500.0, y: 400.0 }),
        max_window_size: Some(egui::Vec2 { x: 500.0, y: 700.0 }),
        icon_data: Some(icon_data),
        ..Default::default()
    };

    eframe::run_native(Box::new(app), native_options);
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
