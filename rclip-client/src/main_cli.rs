use clap::{App, Arg};
use rclip_config;
use std::error::Error;
use std::path::Path;

mod common;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let app = App::new(option_env!("CARGO_PKG_NAME").unwrap_or("Unknown"))
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("Unknown"))
        .author(option_env!("CARGO_PKG_AUTHORS").unwrap_or("Unknown"))
        .about(option_env!("CARGO_PKG_DESCRIPTION").unwrap_or("Unknown"))
        .arg(
            Arg::with_name("host")
                .long("host")
                .help("Server host")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .help("Server port")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("command")
                .long("command")
                .help("READ, WRITE or CLEAR")
                .required(false)
                .possible_values(&["READ", "WRITE", "CLEAR"])
                .default_value("READ")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("text")
                .long("text")
                .help("Text to write to the clipboard server.")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("der-cert-pub")
                .long("der-cert-pub")
                .help("Public DER certificate key")
                .required(false)
                .takes_value(true),
        );

    let run_matches = app.to_owned().get_matches();

    let mut client_config =
        match rclip_config::load_default_config(common::DEFAULT_CONFIG_FILENAME_CLIENT) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Warn: Error parsing configuration file: {}!", e.to_string());
                rclip_config::ClientConfig::default()
            }
        };

    if client_config.certificates.der_cert_pub.is_none() {
        client_config.certificates.der_cert_pub =
            rclip_config::resolve_default_cert_path(rclip_config::DEFAULT_FILENAME_DER_CERT_PUB);
    }

    if let Some(proposed_host) = run_matches.value_of("host") {
        client_config.server.host = Some(proposed_host.to_string());
    }

    if let Some(proposed_port) = run_matches.value_of("port") {
        client_config.server.port = Some(proposed_port.parse::<u16>()?)
    }

    if let Some(key_pub_loc) = run_matches.value_of("der-cert-pub") {
        client_config.certificates.der_cert_pub = Some(key_pub_loc.to_string());
    };

    if client_config.certificates.der_cert_pub.is_none() {
        return Err("Please provide the public certificate argument for --der-cert-pub.".into());
    }

    if let Some(key_loc) = client_config.certificates.der_cert_pub.clone() {
        let key_path = Path::new(&key_loc);

        if !key_path.exists() {
            return Err(format!("The public key file doesn't exists at '{}'!", &key_loc).into());
        }
    }

    let cmd_text_opt = run_matches.value_of("text").map(|i| i.to_string()).or(None);

    let proposed_cmd = run_matches.value_of("command").unwrap_or("READ");

    let clipboard_cmd = match proposed_cmd {
        "READ" => common::ClipboardCmd {
            name: "READ".to_string(),
            text: None,
        },
        "CLEAR" => common::ClipboardCmd {
            name: "CLEAR".to_string(),
            text: Some(String::new()),
        },
        _ => common::ClipboardCmd {
            name: "WRITE".to_string(),
            text: match cmd_text_opt {
                Some(x) => Some(x.to_string()),
                _ => {
                    if let Ok(clipboard_contents) = common::get_clipboard_contents() {
                        Some(clipboard_contents)
                    } else {
                        return Err("Could not acquire clipboard contents.".into());
                    }
                }
            },
        },
    };

    if let (Some(server_host), Some(server_port), Some(der_cert_pub)) = (
        client_config.server.host,
        client_config.server.port,
        client_config.certificates.der_cert_pub,
    ) {
        common::send_cmd(server_host, server_port, der_cert_pub, clipboard_cmd)
    } else {
        Err("Client error! Some required parameters are were not provided: missing public certificate?".into())
    }
}
