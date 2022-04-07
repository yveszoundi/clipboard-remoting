use std::convert::TryFrom;
use std::io;
use std::time::SystemTime;

use rustls::client::ServerCertVerified;
use rustls::ServerName;
use std::error::Error;
use std::fs;
use std::sync::Arc;
use tokio::io::{split, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::rustls::Certificate;
use tokio_rustls::TlsConnector;

const BUFFER_CAP: usize = 512;
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

struct AcceptSpecificCertsVerifier {
    certs: Vec<rustls::Certificate>,
}

impl rustls::client::ServerCertVerifier for AcceptSpecificCertsVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        let presented_cert = end_entity; //&intermediates[0];
        for cert in &self.certs {
            if presented_cert == cert {
                return Ok(rustls::client::ServerCertVerified::assertion());
            }
        }
        return Err(rustls::Error::General("Unknown issuer".to_string()));
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

pub async fn send_cmd(
    server_host: &str,
    port_number: u16,
    key_pub_loc: &str,
    clipboard_cmd: ClipboardCmd,    
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let input = format!("{}", clipboard_cmd);

    let key_pub_bytes =
        fs::read(key_pub_loc)?;

    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(AcceptSpecificCertsVerifier {
            certs: vec![Certificate(key_pub_bytes)],
        }))
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));
    let addr = format!("{}:{}", server_host, port_number);
    let stream = TcpStream::connect(addr.clone()).await?;

    // Just need to resolve a domain, as IP addresses are not supported to use the actual server IP
    // see also https://docs.rs/rustls/latest/rustls/enum.ServerName.html
    let domain = rustls::ServerName::try_from("localhost")
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid dnsname"))?;

    let stream = connector.connect(domain, stream).await?;

    let (mut reader, mut writer) = split(stream);

    writer.write_all(input.as_bytes()).await?;

    let mut response = String::new();

    loop {
        let mut buf_vec = vec![0; BUFFER_CAP];
        let bytes_read = match reader.read(&mut buf_vec).await {
            Ok(n) => match String::from_utf8(buf_vec[0..n].to_vec()) {
                Ok(data) => {
                    response.push_str(&data);
                    n
                }
                Err(e) => {
                    return Err(format!("Cannot read server data; err = {}", e.to_string()).into());
                }
            },
            Err(e) => {
                return Err(format!("Failed to read from socket; err = {}", e.to_string()).into());
            }
        };

        if bytes_read == 0 || bytes_read < BUFFER_CAP {
            break;
        }
    }

    if response.starts_with("SUCCESS:") {
        if input.starts_with("READ:") || input.starts_with("CLEAR:") {
            let clipboard_text = response.chars().skip("SUCCESS:".len()).collect();
            set_clipboard_contents(clipboard_text)?;
        }
    } else {
        return Err(response.into());
    }

    Ok(())
}
