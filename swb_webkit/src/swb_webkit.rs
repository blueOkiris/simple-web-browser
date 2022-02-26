/*
 * Author: Dylan Turner
 * Description: Browser plugin for swb that uses Webkit Gtk
 */

use gtk4::Box;

const NAME: &'static str = "Swb Webkit";

/* Unused plugin functions */

#[no_mangle]
pub fn on_navbar_load(_navbar: &Box) { }

/* Used plugin functions */

#[no_mangle]
pub fn name() -> String {
    String::from(NAME)
}

#[no_mangle]
pub fn on_back_btn_clicked() {
    // TODO: Navigate backwards
}

#[no_mangle]
pub fn on_fwd_btn_clicked() {
    // TODO: Navigate forward
}

#[no_mangle]
pub fn on_change_page(_url: &String) {
    // TODO: Change page
}

#[no_mangle]
pub fn on_refr_btn_clicked() {
    // TODO: Refresh
}
