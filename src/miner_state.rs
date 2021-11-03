/// Defines cli settings to be passed to ethminer
pub struct MinerSettings {
    /// Multiple Url flags are allowed to be specified
    pub url: Vec<Url>,
    pub device_type: Option<DeviceType>,
    /// Display interval in seconds
    pub display_interval: f32,
    /// Path to ethminer bin
    pub bin_path: String,
}

impl Default for MinerSettings {
    fn default() -> Self {
        Self {
            url: vec![Url::default()],
            device_type: None,
            display_interval: 1.0,
            bin_path: "~/Desktop/ethminer/bin/ethminer".to_owned(),
        }
    }
}

impl Clone for MinerSettings {
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
            device_type: self.device_type.clone(),
            display_interval: self.display_interval,
            bin_path: self.bin_path.clone(),
        }
    }
}

impl MinerSettings {
    /// Render settings into valid cli args
    pub fn render(&self) -> String {
        let mut out = String::new();
        match &self.device_type {
            Some(s) => {
                out.push_str(&s.render());
            }
            None => {}
        }
        for url in &self.url {
            out.push_str(&url.render());
            out.push_str(" ");
        }
        out.push_str(" ");
        out
    }
}

#[derive(Clone)]
pub enum DeviceType {
    OpenCl(ClSettings),
    Cuda(CudaSettings),
}

impl DeviceType {
    pub fn render(&self) -> String {
        let mut out = String::new();
        match &self {
            DeviceType::OpenCl(s) => {
                out.push_str("-G ");
                out.push_str(&s.render());
            }
            DeviceType::Cuda(s) => {
                out.push_str("-U ");
                out.push_str(&s.render());
            }
        }
        out.push_str(" ");
        out
    }
}

#[derive(Clone)]
pub struct ClSettings {
    pub global_work: String,
    pub local_work: String,
}

impl ClSettings {
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str("--cl-global-work ");
        out.push_str(&self.global_work.to_string());
        out.push_str(" --cl-local-work ");
        out.push_str(&self.local_work.to_string());
        out
    }
}

#[derive(Clone)]
pub struct CudaSettings {
    pub grid_size: String,
    pub block_size: String,
}

impl CudaSettings {
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str("--cu-grid-size ");
        out.push_str(&self.grid_size.to_string());
        out.push_str(" --cu-block-size ");
        out.push_str(&self.block_size.to_string());
        out
    }
}

/// The The URL is in the form :
///   scheme://[user[.workername][:password]@]hostname:port[/...].
#[derive(Clone)]
pub struct Url {
    pub wallet_address: String,
    pub miner_name: Option<String>,
    pub pool: String,
    pub port: String,
    pub scheme: Scheme,
}

impl Default for Url {
    fn default() -> Self {
        Self {
            wallet_address: "0x03FeBDB6D16B8A19aeCf7c4A777AAdB690F89C3C".to_owned(),
            miner_name: None,
            pool: "us2.ethermine.org".to_string(),
            port: "4444".to_string(),
            scheme: Scheme {
                stratum: Stratum::stratum2,
                transport: Transport::ssl,
            },
        }
    }
}

impl Url {
    fn render(&self) -> String {
        let mut out = String::new();
        out.push_str("-P ");
        out.push_str(&self.scheme.stratum.to_string());
        out.push_str("+");
        out.push_str(&self.scheme.transport.to_string());
        out.push_str("://");
        out.push_str(&self.wallet_address);
        match &self.miner_name {
            Some(s) => {
                out.push_str(".");
                out.push_str(&s);
            }
            None => {}
        }
        out.push_str("@");
        out.push_str(&self.pool);
        out.push_str(":");
        out.push_str(&self.port.to_string());
        out
    }
}

#[derive(Clone)]
pub struct Scheme {
    // 0 1 2 or 3
    pub stratum: Stratum,
    pub transport: Transport,
}

#[derive(ToString, PartialEq, Clone)]
pub enum Stratum {
    stratum,
    stratum1,
    stratum2,
    stratum3,
}

#[derive(ToString, PartialEq, Clone)]
pub enum Transport {
    tcp,
    tls,
    tls12,
    ssl,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_url_render() {
        let url = Url::default();
        println!("{}", url.render());
    }
    #[test]
    fn test_default_settings_render() {
        let settings = MinerSettings::default();
        println!("{}", settings.render());
    }
    #[test]
    fn test_cl_render() {
        let cl = DeviceType::OpenCl(ClSettings {
            local_work: "12".to_string(),
            global_work: "12".to_string(),
        });
        println!("{}", cl.render());

        let cuda = DeviceType::Cuda(CudaSettings {
            grid_size: "32".to_string(),
            block_size: "32".to_string(),
        });
        println!("{}", cuda.render());

        let mut settings = MinerSettings {
            device_type: Some(cuda),
            ..Default::default()
        };
        println!("Cuda cli: {}", settings.render());

        settings = MinerSettings {
            device_type: Some(cl),
            ..Default::default()
        };
        println!("Cl cli: {}", settings.render());
    }

    #[test]
    fn test_mult_urls() {
        let settings = MinerSettings {
            url: vec![Url::default(), Url::default()],
            ..Default::default()
        };
        println!("{}", settings.render());
    }
}
