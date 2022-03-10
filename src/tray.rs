use crate::icon_data::get_icon_argb;
use crate::miner_controller::MinerController;
use crate::miner_settings::*;

use ksni;
use ksni::menu::*;
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;
use tokio::runtime::Handle;

#[cfg(target_os = "linux")]
struct MinerTrayLinux {
    miner_settings: Arc<RwLock<MinerSettings>>,
    miner_controller: Arc<Mutex<MinerController>>,
    tokio_handle: Handle,
}

#[cfg(target_os = "linux")]
impl ksni::Tray for MinerTrayLinux {
    fn icon_name(&self) -> String {
        "ethminer-gui".into()
    }

    fn title(&self) -> String {
        "Mine ether using a gui application".into()
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        vec![
            StandardItem {
                label: "Exit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Start Miner".into(),
                activate: Box::new(|this: &mut Self| {
                    // Sends to miner_controller spawn_tx
                    let mc = this.miner_settings.read().unwrap().clone();
                    this.miner_controller.blocking_lock().spawn_tx.blocking_send(mc).unwrap();
                }),
                ..Default::default()
            }
            .into(),
        ]
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let icon_data = get_icon_argb().to_vec();
        let icon: ksni::Icon = ksni::Icon {
            width: 64,
            height: 64,
            data: icon_data,
        };
        vec![icon]
    }
}

#[cfg(target_os = "linux")]
pub fn start_tray_linux(ms: Arc<RwLock<MinerSettings>>, mc: Arc<Mutex<MinerController>>, tokio_handle: Handle) {
    let service = ksni::TrayService::new(MinerTrayLinux {
        miner_controller: mc,
        miner_settings: ms,
        tokio_handle,
    });
    let handle = service.handle();
    service.spawn();
}

#[cfg(target_os = "windows")]
pub fn start_tray_windows() {
    !todo();
}
