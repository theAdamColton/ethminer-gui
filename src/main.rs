// This makes the compiled windows app not launch with a console window
#![windows_subsystem = "windows"]

mod icon_data;
mod miner_controller;
mod miner_settings;
mod tray;
mod miner_app;

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

use icon_data::get_icon_rgba;

use eframe::{egui, epi};
use std::sync::Arc;

use miner_app::MinerApp;

use tokio;

#[tokio::main]
async fn main() {
    let mut app: MinerApp = MinerApp::default().await;
    // Gets the icon
    let icon: Vec<u8> = get_icon_rgba().to_vec();
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

    app.start_error_listener().await;

    if cfg!(target_os = "linux") {
        tray::start_tray_linux(app.settings.clone(), app.miner_controller.clone());
    } else if cfg!(target_os = "windows") {
    }

    eframe::run_native(Box::new(app), native_options);
}

