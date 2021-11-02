/**
    Adam Colton 2021

 Simple GUI app for ethminer

  _________________________________________
 |file|options|   etherminer-gui     |-|%|X|
 |-----------------------------------------|
 | Settings:    _______________________    |
 ||wallet ad|  |_->_cuda__|_->_opencl__|   |       radio button
 ||pool adre|  | ....collapseable....  |   |       shows either cuda or opencl specific settings
 ||stratum  |  |__________|____________|   |
 ||transport|                              |
 |                                       //|
 |-----------------------------------------|
 |              ________________________   |
 |  |rr   |    |                        |  |
 |  |stop |    |     hr /time graph     |  |
 |             |________________________|  |
 |              ________________________   |
 |  |start|    |                        |  |
 |  |stop |    |     etherminer sout    |  |       expandable upper panel (window with panels)
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
    settings: MinerSettings,
    enabled: bool,
    changed: bool,
}

impl Default for MinerApp {
    fn default() -> Self {
        Self {
            settings: MinerSettings::default(),
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
                ui.horizontal(|ui| {
                    ui.label("Wallet Address");
                });
                ui.end_row();
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::multiline(&mut self.settings.url[0].wallet_address).hint_text("Enter Wallet Address"))
                })
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
