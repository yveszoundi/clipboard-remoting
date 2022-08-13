use serde::de::DeserializeOwned;
use std::error::Error;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;

pub const PROGRAM_GROUP: &str = "rclip";
pub const DEFAULT_SERVER_HOST: &str = "127.0.0.1";
pub const DEFAULT_SERVER_PORT: u16  = 10080;
pub const DEFAULT_FILENAME_DER_CERT_PUB:  &str = "der-cert-pub.der";

#[derive(Deserialize)]
#[serde(default)]
pub struct Server {
    pub host: Option<String>,
    pub port: Option<u16>,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            host: Some(DEFAULT_SERVER_HOST.to_string()),
            port: Some(DEFAULT_SERVER_PORT),
        }
    }
}

#[derive(Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub server: Server,
    pub certificate: ServerCertificate,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server: Server::default(),
            certificate: ServerCertificate::default(),
        }
    }
}

#[derive(Deserialize)]
#[serde(default)]
pub struct ServerCertificate {
    #[serde(rename(deserialize = "der-cert-pub"))]
    pub der_cert_pub: Option<String>,
    #[serde(rename(deserialize = "der-cert-priv"))]
    pub der_cert_priv: Option<String>,
}

impl Default for ServerCertificate {
    fn default() -> Self {
        Self {
            der_cert_pub: None,
            der_cert_priv: None,
        }
    }
}

#[derive(Deserialize)]
#[serde(default)]
pub struct ClientCertificate {
    #[serde(rename(deserialize = "der-cert-pub"))]
    pub der_cert_pub: Option<String>,
}

impl Default for ClientCertificate {
    fn default() -> Self {
        Self {
            der_cert_pub: None,
        }
    }
}

#[derive(Deserialize)]
#[serde(default)]
pub struct ClientConfig {
    pub server: Server,
    pub certificate: ClientCertificate,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server: Server::default(),
            certificate: ClientCertificate::default(),
        }
    }
}

pub fn resolve_default_cert_path(filename: &str) -> Option<String> {
    if let Some(data_dir) = dirs::data_dir() {
        let data_dir_rclip_tcp = data_dir.join(PROGRAM_GROUP);

        if data_dir_rclip_tcp.exists() {
            let pub_cert_file = data_dir_rclip_tcp.join(filename);

            if pub_cert_file.exists() {
                println!("Found certificate data at: {}.", pub_cert_file.display());

                return Some(format!("{}", pub_cert_file.display()));
            }
        }
    }

    None
}

pub fn load_default_config <T> (filename: &str) -> Result<T, Box<dyn Error>> where T: Default + DeserializeOwned {
    if let Some(config_dir) = dirs::config_dir() {
        let config_dir_rclip_tcp = config_dir.join(PROGRAM_GROUP);

        if config_dir_rclip_tcp.exists() {
            let config_client_file = config_dir_rclip_tcp.join(filename);

            if config_client_file.exists() {
                let mut file_config_client = File::open(config_client_file.clone())?;
                let mut config_data = Vec::new();
                file_config_client.read_to_end(&mut config_data)?;
                let config_client: T = toml::from_slice(&config_data)?;
                println!("Loaded configuration data from: {}.", config_client_file.display());

                return Ok(config_client);
            }
        }
    }

    Ok(T::default())
}

