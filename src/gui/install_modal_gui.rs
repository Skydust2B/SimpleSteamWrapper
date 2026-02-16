use slint::ComponentHandle;
use crate::install::install::install_or_update;
use crate::InstallerWindow;

pub fn show_install_modal() {
    let window = InstallerWindow::new().unwrap();

    let window_clone = window.as_weak();
    window.on_yes_clicked(move || {
        install_or_update();
        window_clone.upgrade_in_event_loop(|w| {
            w.hide().unwrap();
        }).unwrap();
    });

    let window_clone = window.as_weak();
    window.on_no_clicked(move || {
        window_clone.upgrade_in_event_loop(|w| {
            w.hide().unwrap();
        }).unwrap();
    });

    window.run().expect("Failed to run window");
}
