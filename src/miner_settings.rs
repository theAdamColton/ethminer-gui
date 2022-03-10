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
            bin_path: "/home/figes/Desktop/ethminer/bin/ethminer".to_owned(),
        }
    }
}

impl std::fmt::Debug for MinerSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MinerSettings").finish()
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
    pub fn render(&self) -> Vec<String> {
        let mut out = Vec::new();
        // For stdout instead of stderr to make reading the output stream easier
        out.push("--stdout".to_string());
        match &self.device_type {
            Some(s) => {
                out.append(&mut s.render());
            }
            None => {}
        }
        for url in &self.url {
            out.append(&mut url.render());
        }
        out
    }
}

#[derive(Clone)]
pub enum DeviceType {
    OpenCl(ClSettings),
    Cuda(CudaSettings),
}

/// For use by egui radio button
impl PartialEq for DeviceType {
    fn eq(&self, other: &Self) -> bool {
        // returns true if the type of the enum is the same,
        // doesn't look at the data contained by the enum
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl DeviceType {
    pub fn render(&self) -> Vec<String> {
        let mut out = Vec::<String>::new();
        match &self {
            DeviceType::OpenCl(s) => {
                out.push("-G".to_string());
                out.append(&mut s.render());
            }
            DeviceType::Cuda(s) => {
                out.push("-U".to_string());
                out.append(&mut s.render());
            }
        }
        out
    }
}

#[derive(Clone)]
pub struct ClSettings {
    pub global_work: String,
    pub local_work: String,
}

impl ClSettings {
    pub fn render(&self) -> Vec<String> {
        let mut out = Vec::new();
        if &self.global_work != "" {
            out.push(format!("--cl-global-work={}", &self.global_work));
        }
        if &self.local_work != "" {
            out.push(format!(" --cl-local-work={}", &self.local_work));
        }
        out
    }
}

#[derive(Clone)]
pub struct CudaSettings {
    pub grid_size: String,
    pub block_size: String,
}

impl CudaSettings {
    pub fn render(&self) -> Vec<String> {
        let mut out = Vec::new();
        if &self.grid_size != "" {
            out.push(format!("--cu-grid-size={}", &self.grid_size));
        }
        if &self.block_size != "" {
            out.push(format!(" --cu-block-size={}", &self.block_size));
        }
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
            scheme: Default::default(),
        }
    }
}

impl Url {
    fn render(&self) -> Vec<String> {
        let mut str_o = String::new();
        str_o.push_str(&self.scheme.stratum.to_string());
        str_o.push_str("+");
        str_o.push_str(&self.scheme.transport.to_string());
        str_o.push_str("://");
        str_o.push_str(&self.wallet_address);
        match &self.miner_name {
            Some(s) => {
                str_o.push_str(".");
                str_o.push_str(&s);
            }
            None => {}
        }
        str_o.push_str("@");
        str_o.push_str(&self.pool);
        str_o.push_str(":");
        str_o.push_str(&self.port.to_string());
        vec!["-P".to_string(), str_o]
    }
}

#[derive(Clone)]
pub struct Scheme {
    // 0 1 2 or 3
    pub stratum: Stratum,
    pub transport: Transport,
}

impl Default for Scheme {
    fn default() -> Self {
        Self {
            stratum: Stratum::stratum,
            transport: Transport::tcp,
        }
    }
}

#[derive(ToString, PartialEq, Clone)]
#[allow(non_camel_case_types)]
pub enum Stratum {
    stratum,
    stratum1,
    stratum2,
    stratum3,
}

#[derive(ToString, PartialEq, Clone)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
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
        println!("{:?}", url.render());
    }
    #[test]
    fn test_default_settings_render() {
        let settings = MinerSettings::default();
        println!("{:?}", settings.render());
    }
    #[test]
    fn test_cl_render() {
        let cl = DeviceType::OpenCl(ClSettings {
            local_work: "12".to_string(),
            global_work: "12".to_string(),
        });
        println!("{:?}", cl.render());

        let cuda = DeviceType::Cuda(CudaSettings {
            grid_size: "32".to_string(),
            block_size: "32".to_string(),
        });
        println!("{:?}", cuda.render());

        let mut settings = MinerSettings {
            device_type: Some(cuda),
            ..Default::default()
        };
        println!("Cuda cli: {:?}", settings.render());

        settings = MinerSettings {
            device_type: Some(cl),
            ..Default::default()
        };
        println!("Cl cli: {:?}", settings.render());
    }

    #[test]
    fn test_mult_urls() {
        let settings = MinerSettings {
            url: vec![Url::default(), Url::default()],
            ..Default::default()
        };
        println!("{:?}", settings.render());
    }
}
