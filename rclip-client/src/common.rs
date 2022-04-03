use std::error::Error;
use std::net::TcpStream;
use std::io::{Read, Write, BufReader};

pub const DEFAULT_SERVER_HOST_STR: &str = "127.0.0.1";
pub const DEFAULT_SERVER_PORT_STR: &str = "10080";

pub struct ClipboardCmd {
    pub name: String,
    pub text: Option<String>,
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

pub fn get_clipboard_contents() -> Result<String, Box<dyn Error + Send + Sync>> {
    use copypasta::{ClipboardContext, ClipboardProvider};
    let mut ctx = ClipboardContext::new()?;
    Ok(format!("{}", ctx.get_contents()?))
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn set_clipboard_contents(clipboard_text: String) -> Result<(), Box<dyn Error + Send + Sync>> {
    use copypasta_ext::prelude::*;
    use copypasta_ext::x11_fork::ClipboardContext;

    let mut ctx = ClipboardContext::new()?;
    ctx.set_contents(clipboard_text)?;
    Ok(())
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn set_clipboard_contents(clipboard_text: String) -> Result<(), Box<dyn Error + Send + Sync>> {
    use copypasta::{ClipboardContext, ClipboardProvider};

    let mut ctx = ClipboardContext::new()?;
    ctx.set_contents(clipboard_text)?;

    Ok(())
}

pub fn send_cmd(server_host: &str, port_number: u16, clipboard_cmd: ClipboardCmd) -> Result<(), Box<dyn Error + Send + Sync>> {
    let input = format!("{}", clipboard_cmd);

    match TcpStream::connect(format!("{}:{}", server_host, port_number)) {
        Ok(mut stream) => {
            let request = input.as_bytes();
            stream.write(request)?;

            let mut reader = BufReader::new(&stream);
            let mut buf = String::new();
            reader.read_to_string(&mut buf).unwrap();

            let response = buf.clone();//from_utf8(buffer)?;

            if response.starts_with("SUCCESS:") {
                if input.starts_with("READ:") {
                    let clipboard_text = response.chars().skip("SUCCESS:".len()).collect();
                    set_clipboard_contents(clipboard_text)?;
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
