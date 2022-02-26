/*
 * Author: Dylan Turner
 * Description: Browser plugin for swb that uses Webkit Gtk
 */

use gtk4::Box;

const NAME: &'static str = "Swb Webkit";

#[no_mangle]
pub fn name() -> String {
    String::from(NAME)
}

#[no_mangle]
pub fn on_back_btn_clicked() {
    println!("Back called from {}", NAME);
}

#[no_mangle]
pub fn on_fwd_btn_clicked() {
    println!("Forward called from {}", NAME);
}

#[no_mangle]
pub fn on_change_page(url: &String) {
    println!("Change page to {} called from {}", url, NAME);
}

#[no_mangle]
pub fn on_refr_btn_clicked() {
    println!("Refresh called from {}", NAME);
}

#[no_mangle]
pub fn on_navbar_load(_navbar: &Box) {
    println!("Navbar Load called from {}", NAME);
}
