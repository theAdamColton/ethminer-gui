mod miner_controller;
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
mod miner_state;

extern crate strum;
#[macro_use]
extern crate strum_macros;

use miner_controller::MinerController;

use eframe::{egui, epi};
use egui::TextStyle;
use miner_state::*;

use std::sync::Arc;
use tokio;
use tokio::sync::Mutex;

pub struct MinerApp<'a> {
    /// Stores the currently used settings
    settings: MinerSettings,
    /// Stores the settings that haven't been applied yet
    temp_settings: MinerSettings,
    miner_controller: Arc<Mutex<MinerController<'static>>>,
    output: &'a Option<Vec<String>>,
}

impl MinerApp<'_> {
    /// Aquires the lock and sends to the spawn channel
    fn run_ethminer(&self) {
        let mc = self.miner_controller.clone();
        tokio::spawn(async move {
            mc.lock()
                .await
                .spawn_tx
                .send(())
                .await
                .expect("Could not send spawn");
        });
    }

    /// Aquires the lock and sends to the kill channel
    fn kill_child_miner(&self) {
        let mc = self.miner_controller.clone();
        tokio::spawn(async move {
            mc.lock()
                .await
                .kill_tx
                .send(())
                .await
                .expect("Could not send kill");
        });
    }

    /// Aquires the lock and sends to the update channel
    fn update_output(&self) {
        let mc = self.miner_controller.clone();
        tokio::spawn(async move {
            mc.lock()
                .await
                .update_tx
                .send(())
                .await
                .expect("Could not send update request");
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
        // The output box
        ui.separator();
        egui::ScrollArea::vertical()
            .stick_to_bottom()
            //            .show(ui, |ui| {
            //                let mut o = String::new();
            //                for x in 0..1000 {
            //                    o.push_str("Beer is the mind killer. ");
            //                }
            //                ui.with_layout(
            //                    egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true),
            //                    |ui| {
            //                        //ui.label(&self.output);
            //                        ui.label(&o);
            //                    },
            //                );
            //            });
            .show(ui, |ui| {
                if let Some(out) = self.output {
                    out.into_iter().for_each(|line| {
                        ui.label(line);
                    });
                }
            });
    }
}

impl Default for MinerApp<'_> {
    fn default() -> Self {
        Self {
            settings: MinerSettings::default(),
            temp_settings: MinerSettings::default(),
            miner_controller: MinerController::new(),
            output: &None,
        }
    }
}

impl Drop for MinerApp<'_> {
    fn drop(&mut self) {
        self.kill_child_miner();
    }
}

impl epi::App for MinerApp<'_> {
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.collapsing("Miner Settings", |ui| {
                ui.collapsing("Pool Settings", |ui| {
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
                                ui.radio_value(
                                    &mut url.scheme.stratum,
                                    Stratum::stratum,
                                    "Stratum",
                                );
                                ui.radio_value(
                                    &mut url.scheme.stratum,
                                    Stratum::stratum1,
                                    "Stratum1",
                                );
                                ui.radio_value(
                                    &mut url.scheme.stratum,
                                    Stratum::stratum2,
                                    "Stratum2",
                                );
                                ui.radio_value(
                                    &mut url.scheme.stratum,
                                    Stratum::stratum3,
                                    "Stratum3",
                                );
                            });
                        });
                    }
                });

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
                        self.temp_settings = self.settings.clone();
                    }
                    if ui.button("Apply").clicked() {
                        self.settings = self.temp_settings.clone();
                        println!("{:?}", &self.settings.render());
                    }
                    if ui.button("Run").clicked() {
                        self.run_ethminer();
                    }
                    if ui.button("Stop").clicked() {
                        self.kill_child_miner();
                    }
                });
            });
            self.show_ethminer_out(ui);
        });
    }

    fn name(&self) -> &str {
        "etherminer-gui"
    }
}

#[tokio::main]
async fn main() {
    let app: MinerApp = MinerApp::default();
    let native_options = eframe::NativeOptions::default();
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
