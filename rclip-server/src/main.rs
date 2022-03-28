use tokio::net::TcpListener;
use tokio::io::AsyncWriteExt;
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
        let clipboard_copy  = clipboard.clone();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            let mut request = String::new();
            let mut err_msg = String::new();

            loop {
                let mut buf = [0; 1024];

                match reader.try_read(&mut buf) {
                    Ok(n) => {

                        if n == 0 {
                            break;
                        }

                        let bytes = &buf[0..n];

                        match &String::from_utf8(bytes.to_vec()) {
                            Ok(data) => {
                                request.push_str(data);
                                n
                            },
                            Err(ex) => {
                                err_msg = format!("ERROR:Cannot decode request. {}.", ex.to_string());
                                0
                            }
                        }
                    },
                    Err(_) => {
                        break;
                    }
                };
            }

            if err_msg.len() == 0 {
                let response = handle_message(&request, clipboard_copy);

                if let Err(e) = writer.write_all(response.as_bytes()).await {
                    return Err(e.to_string());
                }
            } else {
                return Err(err_msg);
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
