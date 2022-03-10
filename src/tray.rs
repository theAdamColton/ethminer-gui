use crate::icon_data::get_icon_argb;
use ksni;
use ksni::menu::*;

#[derive(Debug)]
#[cfg(target_os = "linux")]
struct MinerTrayLinux;

#[cfg(target_os = "linux")]
impl ksni::Tray for MinerTrayLinux {
    fn icon_name(&self) -> String {
        "ethminer-gui".into()
    }

    fn title(&self) -> String {
        "Mine ether using a gui application".into()
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        vec![StandardItem {
            label: "Exit".into(),
            icon_name: "application-exit".into(),
            activate: Box::new(|_| std::process::exit(0)),
            ..Default::default()
        }
        .into(),
//        StandardItem {
//            label: "Start Miner".into(),
//            activate: Box::new(|_| 
//            ..Default::default()
//        }.into(),
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
pub fn start_tray_linux() {
    let service = ksni::TrayService::new(MinerTrayLinux {});
    let handle = service.handle();
    service.spawn();
}

#[cfg(target_os = "windows")]
pub fn start_tray_windows() {
    !todo();
}
