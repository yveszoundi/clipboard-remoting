use clap::{App, Arg};
use std::error::Error;

mod common;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let app = App::new("rclip-client-cli")
        .version("0.0.1")
        .author("Yves Zoundi")
        .about("Clipboard client")
        .arg(
            Arg::with_name("clipboard-program")
                .long("clipboard-program")
                .help("External clipboard wrapper accepting a single input argument.")
                .required(false)
                .takes_value(true)
        ).arg(
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

    let clipboard_program = run_matches.value_of("clipboard-program");
    let proposed_cmd = match run_matches.value_of("command") {
        Some(cmd) => cmd,
        None => "READ"
    };

    let cmd_text_opt = if let Some(xx) = run_matches.value_of("text") {
        Some(xx.to_string())
    } else {
        None
    };    

    if cfg!(any(target_os="freebsd", target_os="netbsd", target_os="openbsd", )) {
        if clipboard_program.is_none() {
            if cmd_text_opt.is_none() && proposed_cmd == "READ" {
                return Err("The clipboard program argument is required for BSD platforms.".into());                
            }
        }
    }

    let clipboard_cmd = match proposed_cmd {
        "READ" => {
            common::ClipboardCmd {
                name: "READ".to_string(),
                text: None,
                clipboard_program: match clipboard_program {
                    Some(x) => Some(x.to_string()),
                    _ => None
                }
            }
        },
        _ => {
            common::ClipboardCmd {
                name: "WRITE".to_string(),
                text: match cmd_text_opt {
                    Some(x) => Some(x.to_string()),
                    _ => {
                        if let Ok(clipboard_contents) = common::get_clipboard_contents(clipboard_program) {
                            Some(clipboard_contents)
                        } else {
                            return Err("Could not acquire clipboard contents".into())
                        }
                    }
                },
                clipboard_program: match clipboard_program {
                    Some(x) => Some(x.to_string()),
                    _ => None
                }
            }
        }
    };

    common::send_cmd(server_host, server_port.parse::<u16>()?, clipboard_cmd)
}
