/**
 * Adam Colton 2021
 *
 * Simple GUI app for ethminer
 */
mod miner_state;

extern crate strum;
#[macro_use]
extern crate strum_macros;



use eframe::{egui, epi};
use miner_state::*;

pub struct MinerApp {
    settings: Settings,
    enabled: bool,
    changed: bool,
}

impl Default for MinerApp {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            enabled: true,
            changed: false,
        }
    }
}

fn main() {
    println!("Hello, world!");
}
