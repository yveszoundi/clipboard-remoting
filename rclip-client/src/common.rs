use std::error::Error;
use std::net::TcpStream;
use std::io::{Read, Write, BufReader};
use std::str::from_utf8;
use copypasta::{ClipboardContext, ClipboardProvider};
use std::process::Command;

pub const DEFAULT_SERVER_HOST_STR: &str = "127.0.0.1";
pub const DEFAULT_SERVER_PORT_STR: &str = "10080";

pub struct ClipboardCmd {
    pub name: String,
    pub text: Option<String>,
    pub clipboard_program: Option<String>,
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

pub fn local_clipboard_contents() -> Result<String, Box<dyn Error + Send + Sync>> {
    let mut ctx = ClipboardContext::new()?;
    Ok(format!("{}", ctx.get_contents()?))
}

pub fn send_cmd(server_host: &str, port_number: u16, clipboard_cmd: ClipboardCmd) -> Result<(), Box<dyn Error + Send + Sync>> {
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
