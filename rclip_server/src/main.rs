use clap::{App, Arg};
use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;
use tokio::io::{split, AsyncReadExt, AsyncWriteExt};

use std::sync::{Arc, Mutex};

use tokio::net::TcpListener;
use tokio_rustls::rustls::{self, Certificate, PrivateKey};

use tokio_rustls::TlsAcceptor;

const EMPTY_CLIPBOARD_TEXT: &str = "";
    
const CMD_READ: &str  = "READ:";
const CMD_WRITE: &str = "WRITE:";
const CMD_CLEAR: &str = "CLEAR:";

const BUFFER_CAP: usize = 512;

const FILENAME_CONFIG_SERVER: &str = "config-server.toml";
const FILENAME_DER_CERT_PRIV: &str = "der-cert-priv.der";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = App::new(option_env!("CARGO_PKG_NAME").unwrap_or("Unknown"))
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("Unknown"))
        .author(option_env!("CARGO_PKG_AUTHORS").unwrap_or("Unknown"))
        .about(option_env!("CARGO_PKG_DESCRIPTION").unwrap_or("Unknown"))
        .arg(
            Arg::with_name("host")
                .long("host")
                .help("IP address to bind to")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .help("Port number")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("der-cert-priv")
                .long("der-cert-priv")
                .help("Private DER certificate key")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("der-cert-pub")
                .long("der-cert-pub")
                .help("Public DER certificate key")
                .required(true)
                .takes_value(true),
        );

    let run_matches = app.to_owned().get_matches();

    let mut server_config = match rclip_config::load_default_config(FILENAME_CONFIG_SERVER) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Warn: Error parsing configuration file: {}!", e.to_string());
            rclip_config::ServerConfig::default()
        }
    };

    if server_config.certificate.der_cert_pub.is_none() {
        server_config.certificate.der_cert_pub =
            rclip_config::resolve_default_cert_path(rclip_config::DEFAULT_FILENAME_DER_CERT_PUB);
    }

    if server_config.certificate.der_cert_priv.is_none() {
        server_config.certificate.der_cert_priv =
            rclip_config::resolve_default_cert_path(FILENAME_DER_CERT_PRIV);
    }

    if let Some(proposed_host) = run_matches.value_of("host") {
        server_config.server.host = Some(proposed_host.to_string());
    }

    if let Some(proposed_port) = run_matches.value_of("port") {
        server_config.server.port = Some(proposed_port.parse::<u16>()?)
    }

    if let Some(key_pub_loc) = run_matches.value_of("der-cert-pub") {
        server_config.certificate.der_cert_pub = Some(key_pub_loc.to_string());
    };

    if let Some(key_priv_loc) = run_matches.value_of("der-cert-priv") {
        server_config.certificate.der_cert_priv = Some(key_priv_loc.to_string());
    };

    if server_config.certificate.der_cert_pub.is_none() {
        return Err("Please provide the public certificate argument for --der-cert-pub.".into());
    }

    if server_config.certificate.der_cert_priv.is_none() {
        return Err("Please provide the private certificate argument for --der-cert-priv.".into());
    }

    if let Some(key_loc) = server_config.certificate.der_cert_priv.clone() {
        let key_path = Path::new(&key_loc);

        if !key_path.exists() {
            return Err(format!("The private key file doesn't exists at '{}'!", &key_loc).into());
        }
    }

    if let Some(key_loc) = server_config.certificate.der_cert_pub.clone() {
        let key_path = Path::new(&key_loc);

        if !key_path.exists() {
            return Err(format!("The public key file doesn't exists at '{}'!", &key_loc).into());
        }
    }

    if let (Some(server_host), Some(server_port), Some(key_priv_loc), Some(key_pub_loc)) = (
        server_config.server.host,
        server_config.server.port,
        server_config.certificate.der_cert_priv,
        server_config.certificate.der_cert_pub,
    ) {
        serve(
            app.get_name(),
            server_host,
            server_port,
            key_priv_loc,
            key_pub_loc,
        )
        .await
    } else {
        Err(
            "Server error! Some required parameters were not provided: missing certificates?"
                .into(),
        )
    }
}

async fn serve(
    app_name: &str,
    host: String,
    port: u16,
    key_priv_loc: String,
    key_pub_loc: String,
) -> Result<(), Box<dyn Error>> {
    let key_priv_bytes = fs::read(key_priv_loc)?;
    let key_pub_bytes = fs::read(key_pub_loc)?;
    let config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![Certificate(key_pub_bytes)], PrivateKey(key_priv_bytes))
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

    let acceptor = TlsAcceptor::from(Arc::new(config));
    let con_string = format!("{}:{}", host, port);
    let listener = TcpListener::bind(con_string.clone()).await?;

    println!("Starting '{}' on at '{}'.", app_name, con_string);

    let clipboard = Arc::new(Mutex::new(String::new()));

    loop {
        let (stream, _) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let clipboard_copy = clipboard.clone();

        tokio::spawn(async move {
            let mut request = String::new();

            match acceptor.accept(stream).await {
                Ok(stream) => {
                    let (mut reader, mut writer) = split(stream);

                    loop {
                        let mut buf_vec = vec![0; BUFFER_CAP];
                        let bytes_read = match reader.read(&mut buf_vec).await {
                            Ok(n) => match String::from_utf8(buf_vec[0..n].to_vec()) {
                                Ok(data) => {
                                    request.push_str(&data);
                                    n
                                }
                                Err(e) => {
                                    return Err(format!(
                                        "Failed to decode request; err = {}",
                                        e.to_string()
                                    )
                                    .into());
                                }
                            },
                            Err(e) => {
                                return Err(format!(
                                    "Failed to read from socket; err = {}",
                                    e.to_string()
                                )
                                .into());
                            }
                        };

                        if bytes_read == 0 || bytes_read < BUFFER_CAP {
                            break;
                        }
                    }

                    let response = handle_message(request, clipboard_copy);

                    if let Err(e) = writer.write_all(response.as_bytes()).await {
                        return Err(e.to_string());
                    }
                }
                Err(e) => {
                    return Err(
                        format!("Error with TLS negotiation; err = {}", e.to_string()).into(),
                    );
                }
            }

            Ok(())
        });
    }
}

fn handle_message(data: String, clipboard: Arc<Mutex<String>>) -> String {
    match clipboard.lock() {
        Ok(mut clipboard_ref) => {
            if data.starts_with(CMD_READ) {
                return format!("SUCCESS:{}", clipboard_ref.as_str());
            } else if data.starts_with(CMD_WRITE) {
                let new_clipboard = &data[CMD_WRITE.len()..];
                *clipboard_ref = new_clipboard.to_string();
                return format!("SUCCESS:{}", new_clipboard);
            } else if data.starts_with(CMD_CLEAR) {
                clipboard_ref.clear();
                return format!("SUCCESS:{}", EMPTY_CLIPBOARD_TEXT);
            } else {
                return format!("ERROR:Unknown message {}", data);
            }
        }
        Err(ex) => {
            return format!("ERROR:Could not acquire clipboard data. {}", ex);
        }
    }
}
