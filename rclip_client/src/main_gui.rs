#![windows_subsystem = "windows"]

use fltk::{
    app, button, dialog, enums, group, input, prelude::*, window, frame, draw
};
use rclip_config;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

mod common;

const SIZE_PACK_SPACING: i32 = 10;
const ROW_HEIGHT: i32        = 40;
const BUTTON_WIDTH: i32      = 80;
const WINDOW_WIDTH: i32      = 430;
const WINDOW_HEIGHT: i32     = 230;
const LABEL_WIDTH: i32       = 150;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let app = app::App::default().with_scheme(app::Scheme::Gleam);

    let wind_title = format!(
        "{} {}",
        option_env!("CARGO_PKG_NAME").unwrap_or("Unknown"),
        option_env!("CARGO_PKG_VERSION").unwrap_or("Unknown")
    );

    let mut wind = window::Window::default()
        .with_size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .center_screen()
        .with_label(&wind_title);
    wind.set_xclass("rclip");
    wind.make_resizable(true);

    let mut host_pack = group::Pack::default()
        .with_pos(SIZE_PACK_SPACING, SIZE_PACK_SPACING)
        .with_size(400, ROW_HEIGHT)
        .with_type(group::PackType::Horizontal);
    host_pack.set_spacing(SIZE_PACK_SPACING);

    let host_frame = frame::Frame::default()
        .with_size(LABEL_WIDTH, ROW_HEIGHT)
        .with_label("Server host")
        .with_align(enums::Align::Inside | enums::Align::Right);

    let host_input_rc = Rc::new(RefCell::new(
        input::Input::default()
            .with_size(200, 20)
    ));

    let client_config =
        match rclip_config::load_default_config(common::DEFAULT_CONFIG_FILENAME_CLIENT) {
            Ok(cfg) => cfg,
            _ => rclip_config::ClientConfig::default(),
        };

    host_input_rc.borrow_mut().set_tooltip("IP address to bind to");

    if let Some(server_host) = client_config.server.host {
        host_input_rc.borrow_mut().set_value(&server_host);
    }

    host_pack.end();

    let mut port_pack = group::Pack::default()
        .with_size(400, ROW_HEIGHT)
        .below_of(&host_pack, SIZE_PACK_SPACING)
        .with_type(group::PackType::Horizontal);

    port_pack.set_spacing(SIZE_PACK_SPACING);
    let port_frame = frame::Frame::default()
        .with_size(LABEL_WIDTH, ROW_HEIGHT)
        .with_label("Server port")
        .with_align(enums::Align::Inside | enums::Align::Right);
    let port_input_rc = Rc::new(RefCell::new(
        input::Input::default()
            .with_size(200, 20)
    ));

    if let Some(server_port) = client_config.server.port {
        let port_number_text = format!("{}", server_port);

        port_input_rc.borrow_mut().set_value(&port_number_text);
    }

    port_input_rc.borrow_mut().set_tooltip("Server port number");
    port_pack.end();

    let mut key_pack = group::Pack::default()
        .with_size(400, ROW_HEIGHT)
        .below_of(&port_pack, SIZE_PACK_SPACING)
        .with_type(group::PackType::Horizontal);
    key_pack.set_spacing(SIZE_PACK_SPACING);
    let key_frame = frame::Frame::default()
        .with_size(LABEL_WIDTH, ROW_HEIGHT)
        .with_label("Public key")
        .with_align(enums::Align::Inside | enums::Align::Right);
    let key_input_rc = Rc::new(RefCell::new(
        input::Input::default()
            .with_size(200, 20)
    ));

    if let Some(pub_key_loc) =
        rclip_config::resolve_default_cert_path(rclip_config::DEFAULT_FILENAME_DER_CERT_PUB)
    {
        key_input_rc.borrow_mut().set_value(&pub_key_loc);
    }

    key_input_rc
        .borrow_mut()
        .set_tooltip("Public DER key path");
    let mut key_button = button::Button::default()
        .with_size(BUTTON_WIDTH, 20)
        .with_label("Browse...");
    key_button.set_callback({
        let input_pub_cert_ref = key_input_rc.clone();

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
    key_pack.end();

    let mut buttons_pack = group::Pack::default()
        .with_size(400, ROW_HEIGHT)
        .below_of(&key_pack, SIZE_PACK_SPACING)
        .with_type(group::PackType::Horizontal);
    buttons_pack.set_spacing(SIZE_PACK_SPACING);

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
        let input_host_ref = host_input_rc.clone();
        let input_port_ref = port_input_rc.clone();
        let input_pub_cert_ref = key_input_rc.clone();
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
        let input_host_ref = host_input_rc.clone();
        let input_port_ref = port_input_rc.clone();
        let input_pub_cert_ref = key_input_rc.clone();
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
        let input_pub_cert_ref = key_input_rc.clone();
        let input_port_ref = port_input_rc.clone();
        let wind_ref = wind.clone();
        let input_host_ref = host_input_rc.clone();

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
        let mut host_pack = host_pack.clone();
        let mut host_frame = host_frame.clone();
        let host_input_rc = host_input_rc.clone();

        let mut port_pack = port_pack.clone();
        let mut port_frame = port_frame.clone();
        let port_input_rc = port_input_rc.clone();

        let mut key_pack = key_pack.clone();
        let mut key_frame = key_frame.clone();
        let key_input_rc = key_input_rc.clone();
        let mut key_button = key_button.clone();

        let mut buttons_pack = buttons_pack.clone();

        let lw = {
            let mut lw = 100;

            lw = std::cmp::max(lw, draw::measure(&host_frame.label(), true).0);
            lw = std::cmp::max(lw, draw::measure(&port_frame.label(), true).0);
            lw = std::cmp::max(lw, draw::measure(&key_frame.label(), true).0);

            lw
        };

        move |wid, ev| match ev {
            enums::Event::Move => {
                wid.redraw();
                true
            },
            enums::Event::Resize => {
                let widw = wid.w() - (SIZE_PACK_SPACING * 2);

                let mut widy = SIZE_PACK_SPACING;
                host_pack.resize(SIZE_PACK_SPACING, widy, widw, ROW_HEIGHT);

                widy += SIZE_PACK_SPACING + ROW_HEIGHT;
                port_pack.resize(SIZE_PACK_SPACING, widy, widw, ROW_HEIGHT);

                widy += SIZE_PACK_SPACING + ROW_HEIGHT;
                key_pack.resize(SIZE_PACK_SPACING, widy, widw, ROW_HEIGHT);

                widy += SIZE_PACK_SPACING + ROW_HEIGHT;
                buttons_pack.resize(SIZE_PACK_SPACING, widy, widw, ROW_HEIGHT);

                widy = SIZE_PACK_SPACING;
                host_frame.resize(SIZE_PACK_SPACING, widy, lw, ROW_HEIGHT);
                host_input_rc.borrow_mut().resize(SIZE_PACK_SPACING * 2 + lw, widy, widw - lw - SIZE_PACK_SPACING, ROW_HEIGHT);

                widy += SIZE_PACK_SPACING + ROW_HEIGHT;
                port_frame.resize(SIZE_PACK_SPACING, widy, lw, ROW_HEIGHT);
                port_input_rc.borrow_mut().resize(SIZE_PACK_SPACING * 2 + lw, widy, widw - lw - SIZE_PACK_SPACING, ROW_HEIGHT);

                widy += SIZE_PACK_SPACING + ROW_HEIGHT;
                key_frame.resize(SIZE_PACK_SPACING, widy, lw, ROW_HEIGHT);
                key_input_rc.borrow_mut().resize(SIZE_PACK_SPACING * 2 + lw, widy, widw - lw - SIZE_PACK_SPACING * 2 - BUTTON_WIDTH, ROW_HEIGHT);
                key_button.resize(widw - BUTTON_WIDTH - SIZE_PACK_SPACING, widy, BUTTON_WIDTH, ROW_HEIGHT);

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

    buttons_pack.end();

    #[cfg(target_os = "macos")]
    {
        use fltk:: menu;

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
    wind.resize(wind.x(), wind.y(), WINDOW_WIDTH + 1, WINDOW_HEIGHT);

    app.run()?;

    Ok(())
}
