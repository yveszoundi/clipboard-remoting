use clap::{App, Arg};
use std::error::Error;
use std::fs;
use std::io;
use tokio::io::{split, AsyncReadExt, AsyncWriteExt};

use std::sync::{Arc, Mutex};

use tokio::net::TcpListener;
use tokio_rustls::rustls::{self, Certificate, PrivateKey};

use tokio_rustls::TlsAcceptor;

const CMD_READ: &str = "READ:";
const CMD_WRITE: &str = "WRITE:";
const CMD_CLEAR: &str = "CLEAR:";
const BUFFER_CAP: usize = 512;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = App::new(option_env!("CARGO_PKG_NAME").unwrap_or("Unknown"))
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("Unknown"))
        .author("Yves Zoundi")
        .about(option_env!("CARGO_PKG_DESCRIPTION").unwrap_or("Unknown"))
        .arg(
            Arg::with_name("host")
                .long("host")
                .help("IP address to bind to")
                .required(false)
                .default_value("127.0.0.1")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .help("Port number")
                .required(false)
                .default_value("10080")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("der-cert-priv")
                .long("der-cert-priv")
                .help("Private DER certificate key")
                .required(true)
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

    let host = match run_matches.value_of("host") {
        Some(proposed_hostname) => proposed_hostname,
        None => "127.0.0.1",
    };

    let port = match run_matches.value_of("port") {
        Some(proposed_port) => proposed_port,
        None => "10080",
    };

    let con_string = format!("{}:{}", host, port);

    let key_pub_bytes = match run_matches.value_of("der-cert-pub") {
        Some(der_cert_pub) => fs::read(der_cert_pub)?,
        None => return Err("Cannot find cert".into()),
    };

    let key_priv_bytes = match run_matches.value_of("der-cert-priv") {
        Some(der_cert_priv) => fs::read(der_cert_priv)?,
        None => return Err("Cannot find cert".into()),
    };

    let config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![Certificate(key_pub_bytes)], PrivateKey(key_priv_bytes))
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
    let acceptor = TlsAcceptor::from(Arc::new(config));

    let listener = TcpListener::bind(con_string).await?;
    let clipboard = Arc::new(Mutex::new("".to_string()));

    println!(
        "Starting '{}' on {} with port {}.",
        app.get_name(),
        host,
        port
    );

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
        Ok(mut old_clipboard) => {
            if data.starts_with(CMD_READ) {
                return format!("SUCCESS:{}", old_clipboard.as_str());
            } else if data.starts_with(CMD_WRITE) {
                let new_clipboard = &data[CMD_WRITE.len()..];
                old_clipboard.clear();
                old_clipboard.push_str(new_clipboard);
                return format!("SUCCESS:{}", new_clipboard);
            } else if data.starts_with(CMD_CLEAR) {
                let new_clipboard = "";
                old_clipboard.clear();
                old_clipboard.push_str(new_clipboard);
                return format!("SUCCESS:{}", new_clipboard);
            } else {
                return format!("ERROR:Unknown message {}.", data);
            }
        }
        Err(ex) => {
            return format!("ERROR:Could not acquire clipboard data. {}", ex);
        }
    }
}
