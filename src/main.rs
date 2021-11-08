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

use eframe::{egui, epi};
use miner_state::*;
use std::io::{self, Write};
use std::process::{Child, Command};

pub struct MinerApp {
    /// Stores the currently used settings
    settings: MinerSettings,
    /// Stores the settings that haven't been applied yet
    temp_settings: MinerSettings,
    child_handle: Option<Child>, // The handle to the ethminer process
    enabled: bool,               // TODO
    changed: bool,               // TODO
}

impl MinerApp {
    fn run_ethminer(&mut self) {
        // Shuts down any already running child process
        self.kill_child_miner();
        println!("{}", &self.settings.bin_path);
        self.child_handle = Some(Command::new(&self.settings.bin_path)
            .current_dir("/home/figes/Desktop/ethminer/")
            //.args(&self.settings.render())
            .args(["-G", "-P", "stratum+tcp://0x03FeBDB6D16B8A19aeCf7c4A777AAdB690F89C3C@us2.ethermine.org:4444"])
            .spawn()
            .expect("Failed to start ethminer!"));
    }

    fn kill_child_miner(&mut self) {
        match self.child_handle.as_mut() {
            Some(x) => {
                x.kill().expect("Failed to kill child process!");
            }
            None => {}
        }
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
}

impl Default for MinerApp {
    fn default() -> Self {
        Self {
            settings: MinerSettings::default(),
            temp_settings: MinerSettings::default(),
            child_handle: None,
            enabled: true,
            changed: false,
        }
    }
}

impl Drop for MinerApp {
    fn drop(&mut self) {
        self.kill_child_miner();
    }
}

impl epi::App for MinerApp {
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
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
                        //self.run_ethminer();
                    }
                });
            })
        });
    }

    fn name(&self) -> &str {
        "etherminer-gui"
    }
}

fn main() {
    let mut app: MinerApp = MinerApp::default();
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
