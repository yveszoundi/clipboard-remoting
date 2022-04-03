use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use clap::{App, Arg};
use std::sync::{Arc, Mutex};

const CMD_READ   : &str  = "READ:";
const BUFFER_CAP : usize = 2048;
const CMD_WRITE  : &str  = "WRITE:";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new(option_env!("CARGO_PKG_NAME").unwrap_or("Unknown"))
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("Unknown"))
        .author("Yves Zoundi")
        .about(option_env!("CARGO_PKG_DESCRIPTION").unwrap_or("Unknown"))
        .arg(
            Arg::with_name("host")
                .long("host")
                .help("Host defaulting to 127.0.0.1")
                .required(false)
                .default_value("127.0.0.1")
                .takes_value(true)
        ).arg(
            Arg::with_name("port")
                .long("port")
                .help("port")
                .required(false)
                .default_value("10080")
                .takes_value(true)
        );

    let run_matches= app.to_owned().get_matches();

    let host = match run_matches.value_of("host") {
        Some(proposed_hostname) => proposed_hostname,
        None => "127.0.0.1"
    };

    let port = match run_matches.value_of("port") {
        Some(proposed_port) => proposed_port,
        None => "10080"
    };

    let con_string = format!("{}:{}", host, port);
    let listener = TcpListener::bind(con_string).await?;
    let clipboard = Arc::new(Mutex::new("".to_string()));

    println!("Starting server on {} with port {}.", host, port);

    loop {
        let (mut socket, _) = listener.accept().await?;
        let clipboard_copy = clipboard.clone();

        tokio::spawn(async move {
            let mut request = String::new();
            let (mut reader, mut writer) = socket.split();

            loop {
                let mut buf_vec = vec![0; BUFFER_CAP];
                let bytes_read = match reader.read(&mut buf_vec).await {
                    Ok(n) => {
                        let data = String::from_utf8(buf_vec[0..n].to_vec());
                        request.extend(data);
                        n
                    },
                    Err(e) => {
                        return Err(format!("Failed to read from socket; err = {}", e.to_string()).into());
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
            } else {
                return format!("ERROR:Unknown message {}.", data);
            }
        },
        Err(ex) =>{
            return format!("ERROR:Could not acquire clipboard data. {}", ex);
        }
    }
}
