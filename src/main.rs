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

pub struct MinerApp {
    /// Stores the currently used settings
    settings: MinerSettings,
    /// Stores the settings that haven't been applied yet
    temp_settings: MinerSettings,
    port_contents: String, // TODO hack that doesnt work with multiple urls
    enabled: bool,
    changed: bool,
}

impl Default for MinerApp {
    fn default() -> Self {
        Self {
            settings: MinerSettings::default(),
            temp_settings: MinerSettings::default(),
            port_contents: String::new(),
            enabled: true,
            changed: false,
        }
    }
}

impl epi::App for MinerApp {
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            ui.collapsing("Miner Settings", |ui| {
                ui.collapsing("Pool Settings", |ui| {
                    for url in &mut self.temp_settings.url {
                        ui.horizontal(|ui| {
                            ui.centered_and_justified(|ui| {
                                ui.label("Wallet Address");
                                ui.add(egui::TextEdit::multiline(&mut url.wallet_address));
                            });
                        });
                        ui.end_row();
                        ui.horizontal(|ui| {
                            ui.centered_and_justified(|ui| {
                                ui.label("Pool Address");
                                ui.add(egui::TextEdit::multiline(&mut url.pool));
                            });
                        });
                        ui.end_row();

                        ui.horizontal(|ui| {
                            ui.centered_and_justified(|ui| {
                                ui.label("Port");
                                let response = ui.add(egui::TextEdit::singleline(&mut url.port));
                            });
                        });
                        ui.end_row();

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

                ui.horizontal(|ui| {
                    ui.label("Etherminer path");
                    ui.add(egui::TextEdit::multiline(&mut self.temp_settings.bin_path));
                });
                ui.end_row();

                ui.horizontal(|ui| {
                    let response = ui.button("Cancel");
                    if response.clicked() {
                        // Cancel temp_settings
                        self.temp_settings = self.settings.clone();
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
    label: String,
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
