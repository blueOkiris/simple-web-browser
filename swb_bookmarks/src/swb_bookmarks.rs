/*
 * Author: Dylan Turner
 * Description: Browser plugin for swb that uses Webkit Gtk
 */

use gtk4::{
    MenuButton, Box, ArrowType, Popover, ScrolledWindow,
    Orientation,
    prelude::{
        BoxExt
    }
};
use cascade::cascade;

const NAME: &'static str = "Swb Bookmarks";
const DEF_MARGIN: i32 = 5;

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
pub fn on_navbar_load(navbar: &Box) {
    let bm_btn = create_bm_menu();
    navbar.append(&bm_btn);

    let sync_btn = create_sync_menu();
    navbar.append(&sync_btn);
}

// Create menu for logging and syncing passwords and bookmarks
fn create_sync_menu() -> MenuButton {
    // Container for data
    let menu_content = cascade!{
        Box::builder()
            .orientation(Orientation::Vertical)
            .hexpand(true).vexpand(true)
            .build();
    };
    let menu_scroller = ScrolledWindow::builder()
        .margin_bottom(DEF_MARGIN).margin_top(DEF_MARGIN)
        .margin_start(DEF_MARGIN).margin_end(DEF_MARGIN)
        .child(&menu_content)
        .build();
    let menu = Popover::builder().autohide(true).child(&menu_scroller).build();

    // TODO: Add control interface

    let sync_menu = MenuButton::builder()
        .label("⇅").margin_start(DEF_MARGIN)
        .hexpand(false).direction(ArrowType::Down)
        .popover(&menu)
        .build();

    sync_menu
}

// Create menu for managing bookmarks
fn create_bm_menu() -> MenuButton {
    // Container for data
    let menu_content = cascade!{
        Box::builder()
            .orientation(Orientation::Vertical)
            .hexpand(true).vexpand(true)
            .build();
    };
    let menu_scroller = ScrolledWindow::builder()
        .margin_bottom(DEF_MARGIN).margin_top(DEF_MARGIN)
        .margin_start(DEF_MARGIN).margin_end(DEF_MARGIN)
        .child(&menu_content)
        .build();
    let menu = Popover::builder().autohide(true).child(&menu_scroller).build();

    // TODO: Load and add bookmarks data to bm menu

    // TODO: Add the control buttons
    

    let bm_menu = MenuButton::builder()
        .label("🕮").margin_start(DEF_MARGIN)
        .hexpand(false).direction(ArrowType::Down)
        .popover(&menu)
        .build();

    bm_menu
}
