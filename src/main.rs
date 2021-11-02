/**
 * Adam Colton 2021
 *
 * Simple GUI app for ethminer
 
   _________________________________________
  |file|options|   etherminer-gui     |-|%|X|
  |-----------------------------------------|
  | Settings:    _______________________    |                       |
  ||wallet ad|  |_->_cuda__|_->_opencl__|   |
  ||pool adre|  | ....collapseable....  |   |
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
  |  |stop |    |     etherminer sout    |  |
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

fn main() {
    println!("Hello, world!");
}
