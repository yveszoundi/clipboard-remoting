use clap::{App, Arg};
use std::error::Error;

mod common;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let app = App::new(option_env!("CARGO_PKG_NAME").unwrap_or("Unknown"))
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("Unknown"))
        .author("Yves Zoundi")
        .about(option_env!("CARGO_PKG_DESCRIPTION").unwrap_or("Unknown"))
        .arg(
            Arg::with_name("server-host")
                .long("server-host")
                .help("Server host")
                .required(false)
                .default_value(common::DEFAULT_SERVER_HOST_STR)
                .takes_value(true)
        ).arg(
            Arg::with_name("server-port")
                .long("server-port")
                .help("Server port")
                .required(false)
                .default_value(common::DEFAULT_SERVER_PORT_STR)
                .takes_value(true)
        ).arg(
            Arg::with_name("command")
                .long("command")
                .help("WRITE to or READ from clipboard server")
                .required(false)
                .possible_values(&["READ", "WRITE"])
                .default_value("READ")
                .takes_value(true)
        ).arg(
            Arg::with_name("text")
                .long("text")
                .help("Text to write to the clipboard server.")
                .required(false)
                .takes_value(true)
        );

    let run_matches= app.to_owned().get_matches();

    let server_host = match run_matches.value_of("server-host") {
        Some(proposed_hostname) => proposed_hostname,
        None => common::DEFAULT_SERVER_HOST_STR
    };

    let server_port = match run_matches.value_of("server-port") {
        Some(proposed_hostname) => proposed_hostname,
        None => common::DEFAULT_SERVER_PORT_STR
    };

    let cmd_text_opt = if let Some(xx) = run_matches.value_of("text") {
        Some(xx.to_string())
    } else {
        None
    };

    let proposed_cmd = match run_matches.value_of("command") {
        Some(cmd) => cmd,
        None => "READ"
    };

    let clipboard_cmd = match proposed_cmd {
        "READ" => {
            common::ClipboardCmd {
                name: "READ".to_string(),
                text: None
            }
        },
        _ => {
            common::ClipboardCmd {
                name: "WRITE".to_string(),
                text: match cmd_text_opt {
                    Some(x) => Some(x.to_string()),
                    _ => {
                        if let Ok(clipboard_contents) = common::get_clipboard_contents() {
                            Some(clipboard_contents)
                        } else {
                            return Err("Could not acquire clipboard contents".into())
                        }
                    }
                },
            }
        }
    };

    common::send_cmd(server_host, server_port.parse::<u16>()?, clipboard_cmd)
}
