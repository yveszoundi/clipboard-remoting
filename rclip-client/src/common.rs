use std::error::Error;
use std::net::TcpStream;
use std::io::{Read, Write, BufReader};
use std::str::from_utf8;

pub const DEFAULT_SERVER_HOST_STR: &str = "127.0.0.1";
pub const DEFAULT_SERVER_PORT_STR: &str = "10080";
const CLIPBOARD_WAIT_TIMER: std::time::Duration = std::time::Duration::from_secs(1);

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

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn get_clipboard_contents(clipboard_app_opt: Option<&str>) -> Result<String, Box<dyn Error + Send + Sync>> {
    use std::process::Command;

    if let Some(clipboard_app) = clipboard_app_opt {
        let output = Command::new(&clipboard_app).output()?;

        if !output.status.success() {
            return Err(format!("Could acquire clipboard contents with program {}!", clipboard_app).into());
        }

        let output_data = output.stdout;
        let clipboard_text_str = std::str::from_utf8(&output_data)?;

        Ok(clipboard_text_str.to_string())
    } else {
        return Err("The clipboard auxiliary program is required!".into());
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn get_clipboard_contents(_: Option<&str>) -> Result<String, Box<dyn Error + Send + Sync>> {
    use copypasta::{ClipboardContext, ClipboardProvider};
    let mut ctx = ClipboardContext::new()?;

    Ok(format!("{}", ctx.get_contents()?))
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn set_clipboard_contents(clipboard_text: String, _: Option<String>) -> Result<(), Box<dyn Error + Send + Sync>> {
    use copypasta::{ClipboardContext, ClipboardProvider};

    let mut ctx = ClipboardContext::new()?;
    ctx.set_contents(clipboard_text.to_string())?;

    std::thread::sleep(CLIPBOARD_WAIT_TIMER);
    ctx.get_contents()?;

    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn set_clipboard_contents(clipboard_text: String, clipboard_app_opt: Option<String>) -> Result<(), Box<dyn Error + Send + Sync>> {
    use std::process::Command;

    if let Some(clipboard_app) = clipboard_app_opt {
        let exec_status = Command::new(&clipboard_app)
            .arg(clipboard_text)
            .status()?;
        if !exec_status.success() {
            return Err(format!("Could not set clipboard contents with program {}!", clipboard_app).into());
        }

        return Ok(())
    }
    
    Err("The clipboard auxiliary program is required!".into())
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
                    set_clipboard_contents(clipboard_text_str.to_string(), clipboard_cmd.clipboard_program)?;
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
