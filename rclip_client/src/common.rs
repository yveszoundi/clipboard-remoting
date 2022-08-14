use rustls::client::ServerCertVerified;
use rustls::{Certificate, ServerName};
use std::convert::TryFrom;
use std::error::Error;
use std::fs;
use std::io::{self, Read, Write};
use std::net;
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;
use copypasta_ext::prelude::*;
use copypasta_ext::x11_fork::ClipboardContext;

pub const DEFAULT_CONFIG_FILENAME_CLIENT: &str = "config-client.toml";

pub struct ClipboardCmd {
    pub name: String,
    pub text: Option<String>,
}

impl std::fmt::Display for ClipboardCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.name.starts_with("READ") {
            write!(f, "READ:")
        } else if self.name.starts_with("CLEAR") {
            write!(f, "CLEAR:")
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
        for cert in &self.certs {
            if end_entity == cert {
                return Ok(rustls::client::ServerCertVerified::assertion());
            }
        }

        return Err(rustls::Error::General(
            "Unknown certificate issuer.".to_string(),
        ));
    }
}

pub fn get_clipboard_contents() -> Result<String, Box<dyn Error + Send + Sync>> {
    let mut ctx: ClipboardContext = ClipboardContext::new()?;

    // Exception under Windows when the clipboard is empty.
    // Need to revisit it at some point.
    let ret = match ctx.get_contents() {
        Ok(data) => data,
        Err(_) => String::new(),
    };

    Ok(format!("{}", ret))
}

pub fn set_clipboard_contents(clipboard_text: String) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut ctx: ClipboardContext = ClipboardContext::new()?;
    ctx.set_contents(clipboard_text)?;

    Ok(())
}

pub fn send_cmd(
    server_host: String,
    port_number: u16,
    key_pub_loc: String,
    clipboard_cmd: ClipboardCmd,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if !Path::new(&key_pub_loc).exists() {
        return Err(format!("Cannot find public key at: {}", key_pub_loc).into());
    }

    let input = format!("{}", clipboard_cmd);
    let key_pub_bytes = fs::read(key_pub_loc)?;

    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(AcceptSpecificCertsVerifier {
            certs: vec![Certificate(key_pub_bytes)],
        }))
        .with_no_client_auth();

    let addr = format!("{}:{}", server_host, port_number);
    println!("Connecting with server at address:'{}'.", addr);

    let request = input.as_bytes();

    // Just need to resolve a domain, as IP addresses are not supported to use the actual server IP.
    // See also https://docs.rs/rustls/latest/rustls/enum.ServerName.html.
    let dns_name = rustls::ServerName::try_from("localhost")
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid dnsname"))?;

    let mut socket = net::TcpStream::connect(addr)?;
    let mut connection = rustls::ClientConnection::new(Arc::new(config), dns_name)?;
    let mut tls = rustls::Stream::new(&mut connection, &mut socket);

    tls.write_all(request)?;

    let mut response = String::new();
    tls.read_to_string(&mut response)?;

    if response.starts_with("SUCCESS:") {
        if input.starts_with("READ:") || input.starts_with("CLEAR:") {
            let mut clipboard_text: String = response.chars().skip("SUCCESS:".len()).collect();

            if clipboard_text.len() == 0 && cfg!(target_os = "windows") {
                clipboard_text.push_str("\0"); // workaround or MS expectation???
            }

            set_clipboard_contents(clipboard_text)?;
        }
    } else {
        return Err(response.into());
    }

    Ok(())
}
