use std::error::Error;
use std::rc::Rc;
use std::cell::RefCell;

use fltk::{
    app, button, prelude::*, group, window, input, dialog
};

mod common;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let app = app::App::default().with_scheme(app::Scheme::Gleam);

    let mut wind = window::Window::default()
        .with_size(210, 150)
        .center_screen()
        .with_label("Clipboard-remoting");
    let wind_clone = wind.clone();
    let wind_copy = wind.clone();
    wind.make_resizable(true);

    let size_pack_spacing = 10;

    let mut input_row = group::Pack::default()
        .with_pos(20, 20)
        .with_size(200, 40)
        .with_type(group::PackType::Horizontal);
    input_row.set_spacing(size_pack_spacing);
    let input_host = Rc::new(RefCell::new(input::Input::default().with_size(100, 20)));
    let input_host_copy = input_host.clone();
    input_host.borrow_mut().set_value(common::DEFAULT_SERVER_HOST_STR);
    let input_port = Rc::new(RefCell::new(input::Input::default().with_size(60, 20)));
    let input_port_copy = input_port.clone();
    input_port.borrow_mut().set_value(common::DEFAULT_SERVER_PORT_STR);
    input_row.end();

    let mut button_row = group::Pack::default()
        .with_size(200, 40)
        .below_of(&input_row, size_pack_spacing)
        .with_type(group::PackType::Horizontal);
    button_row.set_spacing(size_pack_spacing);

    let mut button_send = button::Button::default().with_size(80, 20).with_label("Send");
    let mut button_receive = button::Button::default().with_size(80, 20).with_label("Receive");

    fn send_cmd(host_text: String, port_text: String, cmd_name: &str, cmd_text: Option<String>) -> Result<(), Box<dyn Error>> {
        let clipboard_cmd = common::ClipboardCmd {
            name: cmd_name.to_string(),
            text: cmd_text,
            clipboard_program: None
        };

        let server_port = port_text.parse::<u16>()?;

        match common::send_cmd(host_text.as_str(), server_port, clipboard_cmd) {
            Ok(_) => Ok(()),
            Err(ex) => Err(format!("{}", ex.to_string()).into())
        }
    }

    button_send.set_callback({
        move |_| {
            let port_text = input_port.borrow().value();
            let host_text = input_host.borrow().value();

            if let Ok(clipboard_contents) = common::local_clipboard_contents() {
                let cmd_text_opt = Some(clipboard_contents);

                if let Err(ex) = send_cmd(host_text, port_text, "WRITE", cmd_text_opt) {
                    dialog::alert(wind_clone.x(), wind_clone.y() + wind_clone.height() / 2, ex.to_string().as_str());
                }
            } else {
                dialog::alert(wind_clone.x(), wind_clone.y() + wind_clone.height() / 2, "Could not acquire clipboard contents!");
            }
        }
    });

    button_receive.set_callback({
        move |_| {
            let port_text = input_port_copy.borrow().value();
            let host_text = input_host_copy.borrow().value();

            if let Err(ex) = send_cmd(host_text, port_text, "READ", None) {
                dialog::alert(wind_copy.x(), wind_copy.y() + wind_copy.height() / 2, ex.to_string().as_str());
            }
        }
    });

    button_row.end();

    wind.end();
    wind.show();

    match app.run() {
        Ok(_) => Ok(()),
        Err(ex) => Err(ex.into())
    }
}
