#![windows_subsystem = "windows"]

use fltk::{app, button, dialog, draw, enums, frame, group, input, prelude::*, window};
use rclip_config;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

mod common;

const SIZE_PACK_SPACING: i32 = 10;
const ROW_HEIGHT: i32        = 40;
const BUTTON_WIDTH: i32      = 80;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let app = app::App::default().with_scheme(app::Scheme::Gleam);

    let wind_title = format!(
        "{} {}",
        option_env!("CARGO_PKG_NAME").unwrap_or("Unknown"),
        option_env!("CARGO_PKG_VERSION").unwrap_or("Unknown")
    );

    let mut wind = window::Window::default()
        .with_size(430, 230)
        .center_screen()
        .with_label(&wind_title);
    wind.set_xclass("rclip");
    wind.make_resizable(true);

    let mut group_host = group::Pack::default()
        .with_pos(100, 20)
        .with_size(400, ROW_HEIGHT)
        .with_type(group::PackType::Horizontal);
    group_host.set_spacing(SIZE_PACK_SPACING);
    let input_host = Rc::new(RefCell::new(
        input::Input::default()
            .with_size(200, 20)
            .with_label("Server host"),
    ));

    let client_config =
        match rclip_config::load_default_config(common::DEFAULT_CONFIG_FILENAME_CLIENT) {
            Ok(cfg) => cfg,
            _ => rclip_config::ClientConfig::default(),
        };

    input_host.borrow_mut().set_tooltip("IP address to bind to");

    if let Some(server_host) = client_config.server.host {
        input_host.borrow_mut().set_value(&server_host);
    }

    group_host.end();

    let mut group_port = group::Pack::default()
        .with_size(400, ROW_HEIGHT)
        .below_of(&group_host, SIZE_PACK_SPACING)
        .with_type(group::PackType::Horizontal);
    group_port.set_spacing(SIZE_PACK_SPACING);
    let input_port = Rc::new(RefCell::new(
        input::Input::default()
            .with_size(200, 20)
            .with_label("Server port"),
    ));

    if let Some(server_port) = client_config.server.port {
        let port_number_text = format!("{}", server_port);

        input_port.borrow_mut().set_value(&port_number_text);
    }

    input_port.borrow_mut().set_tooltip("Server port number");
    group_port.end();

    let mut group_pub_cert = group::Pack::default()
        .with_size(400, ROW_HEIGHT)
        .below_of(&group_port, SIZE_PACK_SPACING)
        .with_type(group::PackType::Horizontal);
    group_pub_cert.set_spacing(SIZE_PACK_SPACING);
    let input_pub_cert = Rc::new(RefCell::new(
        input::Input::default()
            .with_size(200, 20)
            .with_label("Public key"),
    ));

    if let Some(pub_key_loc) =
        rclip_config::resolve_default_cert_path(rclip_config::DEFAULT_FILENAME_DER_CERT_PUB)
    {
        input_pub_cert.borrow_mut().set_value(&pub_key_loc);
    }

    input_pub_cert
        .borrow_mut()
        .set_tooltip("Public DER key path");
    let mut button_pub_cert = button::Button::default()
        .with_size(BUTTON_WIDTH, 20)
        .with_label("Browse...");
    button_pub_cert.set_callback({
        let input_pub_cert_ref = input_pub_cert.clone();

        move |_| {
            let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseFile);
            dlg.set_title("Select public key");
            dlg.show();

            let selected_filename = dlg.filename();

            if !selected_filename.as_os_str().is_empty() {
                let path_name = dlg.filename().display().to_string();
                input_pub_cert_ref.borrow_mut().set_value(&path_name);
            }
        }
    });
    group_pub_cert.end();

    let mut group_buttons = group::Pack::default()
        .with_size(400, ROW_HEIGHT)
        .below_of(&group_pub_cert, SIZE_PACK_SPACING)
        .with_type(group::PackType::Horizontal);
    group_buttons.set_spacing(SIZE_PACK_SPACING);

    let mut button_receive = button::Button::default()
        .with_size(BUTTON_WIDTH, 20)
        .with_label("Read");
    let mut button_send = button::Button::default()
        .with_size(BUTTON_WIDTH, 20)
        .with_label("Write");
    let mut button_clear = button::Button::default()
        .with_size(BUTTON_WIDTH, 20)
        .with_label("Clear");

    fn send_cmd(
        host_text: String,
        port_text: String,
        key_pub_der: String,
        cmd_name: &str,
        cmd_text: Option<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let clipboard_cmd = common::ClipboardCmd {
            name: cmd_name.to_string(),
            text: cmd_text,
        };

        let server_port = port_text.parse::<u16>()?;

        match common::send_cmd(host_text, server_port, key_pub_der, clipboard_cmd) {
            Ok(_) => Ok(()),
            Err(ex) => Err(format!("{}", ex.to_string()).into()),
        }
    }

    button_send.set_callback({
        let input_host_ref = input_host.clone();
        let input_port_ref = input_port.clone();
        let input_pub_cert_ref = input_pub_cert.clone();
        let wind_ref = wind.clone();

        move |_| {
            let port_text = input_port_ref.borrow().value();
            let host_text = input_host_ref.borrow().value();
            let cert_path = input_pub_cert_ref.borrow().value();

            if let Ok(clipboard_contents) = common::get_clipboard_contents() {
                let cmd_text_opt = Some(clipboard_contents);

                if let Err(ex) = send_cmd(host_text, port_text, cert_path, "WRITE", cmd_text_opt) {
                    dialog::alert(
                        wind_ref.x(),
                        wind_ref.y() + wind_ref.height() / 2,
                        ex.to_string().as_str(),
                    );
                }
            } else {
                dialog::alert(
                    wind_ref.x(),
                    wind_ref.y() + wind_ref.height() / 2,
                    "Could not acquire clipboard contents!",
                );
            }
        }
    });

    button_clear.set_callback({
        let input_host_ref = input_host.clone();
        let input_port_ref = input_port.clone();
        let input_pub_cert_ref = input_pub_cert.clone();
        let wind_ref = wind.clone();

        move |_| {
            let port_text = input_port_ref.borrow().value();
            let host_text = input_host_ref.borrow().value();
            let cmd_text_opt = Some(String::new());
            let cert_path = input_pub_cert_ref.borrow().value();

            if let Err(ex) = send_cmd(host_text, port_text, cert_path, "CLEAR", cmd_text_opt) {
                dialog::alert(
                    wind_ref.x(),
                    wind_ref.y() + wind_ref.height() / 2,
                    ex.to_string().as_str(),
                );
            }
        }
    });

    button_receive.set_callback({
        let input_pub_cert_ref = input_pub_cert.clone();
        let input_port_ref = input_port.clone();
        let wind_ref = wind.clone();
        let input_host_ref = input_host.clone();

        move |_| {
            let port_text = input_port_ref.borrow().value();
            let host_text = input_host_ref.borrow().value();
            let cert_path = input_pub_cert_ref.borrow().value();
            let wind_ref = wind_ref.clone();

            if let Err(ex) = send_cmd(host_text, port_text, cert_path, "READ", None) {
                dialog::alert(
                    wind_ref.x(),
                    wind_ref.y() + wind_ref.height() / 2,
                    ex.to_string().as_str(),
                );
            }
        }
    });

    wind.handle({
        let mut wids = [
            group_host.clone(),
            group_port.clone(),
            group_pub_cert.clone(),
            group_buttons.clone(),
        ];

        move |wid, ev| match ev {
            enums::Event::Resize => {

                let label_width = {
                    let mut lw = 0;

                    for wid in wids.iter() {
                        if let Some(wid_child_ref) = wid.child(0) {
                            let (wid_label_width, _) = draw::measure(&wid_child_ref.label(), true);
                            lw = std::cmp::max(lw, wid_label_width);
                        }
                    }

                    lw
                };

                let w = wid.w() - label_width - (SIZE_PACK_SPACING * 2);
                let mut y = SIZE_PACK_SPACING;
                let n = wids.len();
                let fw = w - BUTTON_WIDTH - SIZE_PACK_SPACING;

                for i in 0..n {
                    let wid_ref = &mut wids[i];
                    wid_ref.resize(SIZE_PACK_SPACING + label_width, y, w, ROW_HEIGHT);
                    y += SIZE_PACK_SPACING + ROW_HEIGHT;
                }

                for i in 0..n {
                    let wid_ref = &mut wids[i];

                    if i != n - 1 {
                        let k = wid_ref.children();
                        let mut child_x = label_width + SIZE_PACK_SPACING;

                        for j in 0..k {
                            if let Some(mut child) = wid_ref.child(j) {
                                let child_w = if j == 0 {
                                    fw
                                } else {
                                    BUTTON_WIDTH
                                };

                                child.resize(wid_ref.x(), wid_ref.y(), child_w, ROW_HEIGHT);
                                child_x = child_x + child_w + SIZE_PACK_SPACING;
                            }
                        }
                    }
                }

                true
            }
            _ => {
                if app::event_state().is_empty() && app::event_key() == enums::Key::Escape {
                    true
                } else {
                    false
                }
            }
        }
    });

    group_buttons.end();

    #[cfg(target_os = "macos")]
    {
        use fltk::menu;

        menu::mac_set_about({
            let wind_ref = wind.clone();

            move || {
                let dialog_width = 250;
                let dialog_height = 80;
                let dialog_xpos = wind_ref.x() + (wind_ref.w() / 2) - (dialog_width / 2);
                let dialog_ypos = wind_ref.y() + (wind_ref.h() / 2) - (dialog_height / 2);
                let win_title = format!(
                    "{} {}",
                    "About",
                    option_env!("CARGO_PKG_NAME").unwrap_or("Unknown")
                );

                let mut win = window::Window::default()
                    .with_size(dialog_width, dialog_height)
                    .with_pos(dialog_xpos, dialog_ypos)
                    .with_label(&win_title);

                let dialog_text = format!(
                    "{}\n{} {}",
                    option_env!("CARGO_PKG_DESCRIPTION").unwrap_or("Unknown"),
                    "Version",
                    option_env!("CARGO_PKG_VERSION").unwrap_or("Unknown")
                );

                frame::Frame::default_fill()
                    .with_label(&dialog_text)
                    .with_align(enums::Align::Center | enums::Align::Inside);

                win.end();
                win.make_modal(true);
                win.show();

                while win.shown() {
                    app::wait();
                }
            }
        });
    }

    wind.end();
    wind.show();

    let cb = {
        let mut wind_ref = wind.clone();

        move |_| {
            wind_ref.resize(wind_ref.x(), wind_ref.y(), 400, 230);
        }
    };
    app::add_timeout3(0.01, cb);

    app.run()?;

    Ok(())
}
