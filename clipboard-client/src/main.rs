use clap::{App, Arg};
use std::error::Error;
use std::net::TcpStream;
use std::io::{Read, Write, BufReader};
use std::str::from_utf8;
use copypasta::{ClipboardContext, ClipboardProvider};
use std::process::Command;

pub struct ClipboardCmd {
    name: String,
    text: Option<String>,
    clipboard_program: Option<String>,
}

impl std::fmt::Display for ClipboardCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.name.starts_with("READ") {
            write!(f, "READ:")
        } else {
            if let Some(txt) = &self.text {
                write!(f, "WRITE:{}", txt)
            } else {
                write!(f, "WRITE:")
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let app = App::new("clipboard-client")
        .version("0.0.1")
        .author("Yves Zoundi")
        .about("Clipboard client")
        .arg(
            Arg::with_name("clipboard-program")
                .long("clipboard-program")
                .help("External clipboard application such as xclip (i.e. /usr/bin/xclip).")
                .required(false)
                .takes_value(true)
        ).arg(
            Arg::with_name("server-host")
                .long("server-host")
                .help("Server host")
                .required(false)
                .default_value("localhost")
                .takes_value(true)
        ).arg(
            Arg::with_name("server-port")
                .long("server-port")
                .help("Server port")
                .required(false)
                .default_value("10080")
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
        None => "localhost"
    };

    let server_port = match run_matches.value_of("server-port") {
        Some(proposed_hostname) => proposed_hostname,
        None => "10080"
    };

    let clipboard_program = run_matches.value_of("clipboard-program");
    let proposed_cmd = match run_matches.value_of("command") {
        Some(cmd) => cmd,
        None => "READ"
    };

    let mut cmd_text_opt = if let Some(xx) = run_matches.value_of("text") {
        Some(xx.to_string())
    } else {
        None
    };

    if proposed_cmd == "WRITE" && cmd_text_opt.is_none() {
        let mut ctx = ClipboardContext::new()?;
        let v = format!("{}", ctx.get_contents()?);
        cmd_text_opt = Some(v);
    }

    let clipboard_cmd = match proposed_cmd {
        "READ" => {
            ClipboardCmd {
                name: "READ".to_string(),
                text: None,
                clipboard_program: match clipboard_program {
                    Some(x) => Some(x.to_string()),
                    _ => None
                }
            }
        },
        _ => {
            ClipboardCmd {
                name: "WRITE".to_string(),
                text: match cmd_text_opt {
                    Some(x) => Some(x.to_string()),
                    _ => None
                },
                clipboard_program: match clipboard_program {
                    Some(x) => Some(x.to_string()),
                    _ => None
                }
            }
        }
    };

    run(server_host, server_port.parse::<u16>()?, clipboard_cmd)
}

fn run(server_host: &str, port_number: u16, clipboard_cmd: ClipboardCmd) -> Result<(), Box<dyn Error + Send + Sync>> {
    let input = format!("{}", clipboard_cmd);

    match TcpStream::connect(format!("{}:{}", server_host, port_number)) {
        Ok(mut stream) => {
            let request = input.as_bytes();
            stream.write(request)?;

            let mut reader =BufReader::new(&stream);
            let mut buffer: Vec<u8> = Vec::new();

            reader.read_to_end(&mut buffer)?;
            
            let buffer = &buffer.into_iter().filter(|&i| i != 0).collect::<Vec<u8>>();
            let response = from_utf8(buffer)?;

            if response.starts_with("SUCCESS:") {
                if input.starts_with("READ:") {
                    let clipboard_text = &buffer["SUCCESS:".len()..];
                    let clipboard_text_str = std::str::from_utf8(clipboard_text)?;

                    match clipboard_cmd.clipboard_program {
                        None => {
                            let mut ctx = ClipboardContext::new()?;
                            ctx.set_contents(clipboard_text_str.to_string())?;
                        },
                        Some(clipboard_app) => {
                            let exec_status = Command::new(&clipboard_app)
                                .arg(clipboard_text_str)
                                .status()?;
                            if !exec_status.success() {
                                return Err(format!("Could not copy to clipboard with program {}", clipboard_app).into());
                            }
                        }
                    }
                }
            } else {
                return Err(response.into());
            }
        },
        Err(ex) => {
            return Err(format!("Could not connect to clipboard server at '{}:{}'! {}", server_host, port_number, ex.to_string()).into());
        }
    }

    Ok(())
}
