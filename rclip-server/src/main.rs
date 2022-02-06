use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use clap::{App, Arg};
use std::sync::{Arc, Mutex};

const CMD_READ  : &str = "READ:";
const CMD_WRITE : &str = "WRITE:";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new("rclip-server")
        .version("0.0.1")
        .author("Yves Zoundi")
        .about("Clipboard server")
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
            let mut buf_vec = vec![0; 4096];

            loop {
                match socket.read(&mut buf_vec).await {
                    Ok(n) => {
                        n
                    },
                    Err(e) => {
                        return Err(format!("Failed to read from socket; err = {}", e.to_string()).into());
                    }
                };

                buf_vec = buf_vec.into_iter().filter(|&i| i != 0).collect::<Vec<u8>>();

                match std::str::from_utf8(&buf_vec)  {
                    Ok(request) => {
                        let response = handle_message(request, clipboard_copy);

                        if let Err(e) = socket.write_all(response.as_bytes()).await {
                            return Err(e.to_string());
                        }
                    },
                    Err(e) => return Err(format!("Could not read request data. {}", e.to_string()))
                }

                break;
            }

            Ok(())
        });
    }
}

fn handle_message(msg: &str, clipboard: Arc<Mutex<String>>) -> String {
    match clipboard.lock() {
        Ok(mut old_clipboard) => {
            if msg.starts_with(CMD_READ) {
                return format!("SUCCESS:{}", old_clipboard.as_str());
            } else if msg.starts_with(CMD_WRITE) {
                let new_clipboard = &msg[CMD_WRITE.len()..];
                old_clipboard.clear();
                old_clipboard.push_str(new_clipboard);
                return format!("SUCCESS:{}", new_clipboard);
            } else {
                return format!("ERROR:Unknown message {}.", msg);
            }
        },
        Err(ex) =>{
            return format!("ERROR:Could not acquire clipboard data. {}", ex);
        }
    }
}
