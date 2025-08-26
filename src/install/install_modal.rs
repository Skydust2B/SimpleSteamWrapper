use std::cell::RefCell;
use std::rc::Rc;
use slint::ComponentHandle;
use crate::install::install::install_or_update;
use crate::InstallerWindow;

pub fn show_install_modal() {
    let window = Rc::new(RefCell::new(InstallerWindow::new().unwrap()));

    let window_clone = window.clone();
    window.borrow().on_yes_clicked(move || {
        install_or_update();
        window_clone.borrow().hide().unwrap();
    });

    let window_clone = window.clone();
    window.borrow().on_no_clicked(move || {
        window_clone.borrow().hide().unwrap();
    });

    window.borrow().run().expect("Failed to run window");
}
