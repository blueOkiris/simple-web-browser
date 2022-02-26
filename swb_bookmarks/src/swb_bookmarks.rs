/*
 * Author: Dylan Turner
 * Description: Browser plugin for swb that uses Webkit Gtk
 */

use gtk::{
    MenuButton, Box, ArrowType, Popover, ScrolledWindow, Button,
    Orientation, Frame,
    prelude::{
        ContainerExt, ButtonExt, WidgetExt
    }
};
use cascade::cascade;

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

    // TODO: Add control interface dependent on whether synced in or not
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
        .popover(&menu)
        .build();

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

    // TODO: Load and add bookmarks data to bm menu
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
        .build();

    bm_menu
}
