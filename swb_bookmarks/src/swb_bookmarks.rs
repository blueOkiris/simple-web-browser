/*
 * Author: Dylan Turner
 * Description: Browser plugin for swb that uses Webkit Gtk
 */

mod sync;
mod config;

use gtk::{
    MenuButton, Box, ArrowType, Popover, ScrolledWindow, Button,
    Orientation, Frame,
    prelude::{
        ContainerExt, ButtonExt, WidgetExt
    }
};
use cascade::cascade;
use crate::config::Config;

const NAME: &'static str = "Swb Bookmarks";
const DEF_MARGIN: i32 = 5;
const POPOVER_WIDTH: i32 = 400;
const POPOVER_HEIGHT: i32 = 400;

/* Unused plugin functions */

#[no_mangle]
pub fn on_back_btn_clicked() { }
#[no_mangle]
pub fn on_fwd_btn_clicked() { }
#[no_mangle]
pub fn on_change_page(_url: &String) { }
#[no_mangle]
pub fn on_refr_btn_clicked() { }
#[no_mangle]
pub fn on_window_content_load(_content: &Box) { }

/* Used plugin functions */

#[no_mangle]
pub fn name() -> String {
    String::from(NAME)
}

#[no_mangle]
pub fn on_navbar_load(navbar: &Box) {
    let bm_btn = create_bm_menu();
    navbar.add(&bm_btn);

    let sync_btn = create_sync_menu();
    navbar.add(&sync_btn);
}

// Create menu for logging and syncing passwords and bookmarks
fn create_sync_menu() -> MenuButton {
    // Container for data
    let menu_content = Box::builder()
        .orientation(Orientation::Vertical)
        .margin_bottom(DEF_MARGIN).margin_top(DEF_MARGIN)
        .margin_start(DEF_MARGIN).margin_end(DEF_MARGIN)
        .build();
    let menu = Popover::builder()
        .width_request(POPOVER_WIDTH).height_request(POPOVER_HEIGHT)
        .child(&menu_content)
        .build();

    // Sync gets managed in on click later
    let bm_box = Box::builder()
        .hexpand(true).vexpand(true).orientation(Orientation::Vertical)
        .build();
    let bm_scroller = ScrolledWindow::builder()
        .hexpand(true).vexpand(true)
        .child(&bm_box)
        .build();
    let bm_frame = Frame::builder()
        .label("Sync:").hexpand(true).vexpand(true)
        .child(&bm_scroller)
        .build();
    menu_content.add(&bm_frame);

    menu.show_all();
    menu.hide();

    let sync_menu = MenuButton::builder()
        .label("⇅").margin_start(DEF_MARGIN)
        .hexpand(false).direction(ArrowType::Down)
        .tooltip_text("Sync Menu")
        .popover(&menu)
        .build();
    sync_menu.connect_clicked(|_btn| {
        let cfg = Config::get_global();
        if cfg.stay_logged_in {
            // TODO: Try log in and show sign out button instead
            return;
        }
        
        // TODO: Set up menu for login/register
    });

    sync_menu
}

// Create menu for managing bookmarks
fn create_bm_menu() -> MenuButton {
    // Container for data
    let menu_content = Box::builder()
        .orientation(Orientation::Vertical)
        .margin_bottom(DEF_MARGIN).margin_top(DEF_MARGIN)
        .margin_start(DEF_MARGIN).margin_end(DEF_MARGIN)
        .build();
    let menu = Popover::builder()
        .width_request(POPOVER_WIDTH).height_request(POPOVER_HEIGHT)
        .child(&menu_content)
        .build();

    // Later these get bookmarks filled in
    let bm_box = Box::builder()
        .hexpand(true).vexpand(true).orientation(Orientation::Vertical)
        .build();
    let bm_scroller = ScrolledWindow::builder()
        .hexpand(true).vexpand(true)
        .child(&bm_box)
        .build();
    let bm_frame = Frame::builder()
        .label("Bookmarks:").hexpand(true).vexpand(true)
        .child(&bm_scroller)
        .build();
    menu_content.add(&bm_frame);

    // Add control buttons
    let add_btn = cascade! {
        Button::builder() // Can't use with_label here: crashes w/ gtk::init()
            .label("Add Bookmark")
            .margin_top(DEF_MARGIN).margin_bottom(DEF_MARGIN)
            .build();
            ..connect_clicked(move |_btn| {
                // TODO: Add bookmarks
            });
    };
    menu_content.add(&add_btn);

    let edit_btn = cascade! {
        Button::builder() // Can't use with_label here: crashes w/ gtk::init()
            .label("Edit Bookmark")
            .margin_bottom(DEF_MARGIN)
            .build();
            ..connect_clicked(move |_btn| {
                // TODO: Edit bookmarks
            });
    };
    menu_content.add(&edit_btn);

    let add_fldr_btn = cascade! {
        Button::builder() // Can't use with_label here: crashes w/ gtk::init()
            .label("Add Folder")
            .margin_bottom(DEF_MARGIN)
            .build();
            ..connect_clicked(move |_btn| {
                // TODO: Add folder
            });
    };
    menu_content.add(&add_fldr_btn);

    let rm_btn = cascade! {
        Button::builder() // Can't use with_label here: crashes w/ gtk::init()
            .label("Remove Item")
            .build();
            ..connect_clicked(move |_btn| {
                // TODO: Remove bookmark or folder
            });
    };
    menu_content.add(&rm_btn);

    menu.show_all();
    menu.hide();

    let bm_menu = MenuButton::builder()
        .label("🕮").margin_start(DEF_MARGIN)
        .direction(ArrowType::Down).popover(&menu)
        .tooltip_text("Bookmarks Menu")
        .build();
    bm_menu.connect_clicked(|_btn| {
        // TODO: Load files from bookmark into bm_box
    });

    bm_menu
}
