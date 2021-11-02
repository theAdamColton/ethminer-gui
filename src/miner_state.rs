// Strum contains all the trait definitions

#[macro_use]

/// Defines cli settings to be passed to ethminer
pub struct Settings {
    ///   Multiple Pools are allowed to be specified
    pub url: Vec<Url>,
    pub device_type: Option<DeviceType>,
    /// Display interval in seconds
    pub display_interval: f32,
    /// Path to ethminer bin
    pub bin_path: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            url: vec![Url::default()],
            device_type: None,
            display_interval: 1.0,
            bin_path: "~/Desktop/ethminer/bin/ethminer".to_owned(),
        }
    }
}

impl Settings {
    /// Make this settings into a valid cli args call
    fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.bin_path);
        out.push_str(" ");
        out.to_owned()
    }
}

/// The The URL is in the form :
///   scheme://[user[.workername][:password]@]hostname:port[/...].
///
pub struct Url {
    pub wallet_address: String,
    pub miner_name: Option<String>,
    pub pool: String,
    pub port: u32,
    pub scheme: Scheme,
}

impl Default for Url {
    fn default() -> Self {
        Self {
            wallet_address: "0x03FeBDB6D16B8A19aeCf7c4A777AAdB690F89C3C".to_owned(),
            miner_name: None,
            pool: "us2.ethermine.org".to_owned(),
            port: 4444,
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

pub struct Scheme {
    // 0 1 2 or 3
    pub stratum: Stratum,
    pub transport: Transport,
}

#[derive(ToString)]
pub enum Stratum {
    stratum,
    stratum1,
    stratum2,
    stratum3,
}

#[derive(ToString)]
pub enum Transport {
    tcp,
    tls,
    tls12,
    ssl,
}

pub enum DeviceType {
    OpenCl(ClSettings),
    Cuda(CudaSettings),
}

pub struct ClSettings {
    pub global_work: u32,
    pub local_work: u32,
}

pub struct CudaSettings {
    pub grid_size: u32,
    pub block_size: u32,
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_url_render() {
        let url = Url::default();
        println!("{}", url.render());
    }
}

