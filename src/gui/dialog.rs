use slint::{ComponentHandle, SharedString};
use crate::SimpleDialog;

pub fn show_message_dialog(text: &str) {
    let message = text.to_string();
    let _ = slint::invoke_from_event_loop(move || {
        let window = SimpleDialog::new().unwrap();
        window.set_text(SharedString::from(message));
        window.on_ok_clicked({
            let weak_window = window.as_weak();
            move || {
                weak_window.unwrap().hide().unwrap();
            }
        });
        window.show().unwrap();
    });
}
