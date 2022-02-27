/*
 * Author: Dylan Turner
 * Description: Browser plugin for swb that uses Webkit Gtk
 */

mod sync;
mod config;

use gtk::{
    MenuButton, Box, Popover, ScrolledWindow, Frame,
    Entry, Label, CheckButton, Button,
    Orientation, ArrowType, Align, InputPurpose,
    prelude::{
        ContainerExt, ButtonExt, WidgetExt, BoxExt,
    }, traits::MenuButtonExt
};
use cascade::cascade;
use crate::config::Config;

const NAME: &'static str = "Swb Bookmarks";
const DEF_MARGIN: i32 = 5;
const POPOVER_WIDTH: i32 = 400;
const POPOVER_HEIGHT: i32 = 200;

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
    sync_menu.connect_clicked(move |menu_btn| {
        // Reset the view
        for child in bm_box.children().clone() {
            bm_box.remove(&child);
        }

        let cfg = Config::get_global();
        if cfg.stay_logged_in {
            // TODO: Try log in first
            // if login succeeded {
                let log_out_btn = Button::builder()
                    .label("Sign Out")
                    .margin_bottom(DEF_MARGIN).margin_top(DEF_MARGIN)
                    .margin_start(DEF_MARGIN).margin_end(DEF_MARGIN)
                    .hexpand(true).vexpand(true)
                    .build();

                let menu_btn_clone = menu_btn.clone();
                log_out_btn.connect_clicked(move |_btn| {
                    menu_btn_clone.popover().unwrap().hide();
                });

                bm_box.pack_start(&log_out_btn, true, true, 0);
                bm_box.show_all();
                return;
            //}
        }
        
        let email_hbox = Box::builder()
            .orientation(Orientation::Horizontal)
            .hexpand(true).margin_bottom(DEF_MARGIN)
            .margin_start(DEF_MARGIN).margin_end(DEF_MARGIN)
            .build();
        let email_label = Label::builder()
            .label("Email:         ")
            .margin_end(DEF_MARGIN).halign(Align::Start)
            .build();
        let email = Entry::builder().hexpand(true).build();
        email_hbox.pack_start(&email_label, false, false, 0);
        email_hbox.pack_start(&email, true, true, 0);
        bm_box.pack_start(&email_hbox, false, false, 0);

        let pword_hbox = Box::builder()
            .orientation(Orientation::Horizontal)
            .hexpand(true).margin_bottom(DEF_MARGIN)
            .margin_start(DEF_MARGIN).margin_end(DEF_MARGIN)
            .build();
        let pword_label = Label::builder()
            .label("Password:").margin_end(DEF_MARGIN).halign(Align::Start)
            .build();
        let pword = Entry::builder().hexpand(true).visibility(false).build();
        pword_hbox.pack_start(&pword_label, false, false, 0);
        pword_hbox.pack_start(&pword, true, true, 0);
        bm_box.pack_start(&pword_hbox, false, false, 0);

        let remember = CheckButton::builder()
            .label("Stay logged in:").margin_bottom(DEF_MARGIN)
            .hexpand(true).halign(Align::Center)
            .build();
        bm_box.pack_start(&remember, false, false, 0);

        let btn_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .hexpand(true).margin_bottom(DEF_MARGIN)
            .margin_start(DEF_MARGIN).margin_end(DEF_MARGIN)
            .build();
        let reg_btn = Button::builder()
            .label("Register").margin_end(DEF_MARGIN)
            .build();
        let login_btn = Button::builder().label("Login   ").build();
        btn_box.pack_start(&reg_btn, true, true, 0);
        btn_box.pack_start(&login_btn, true, true, 0);
        bm_box.pack_start(&btn_box, false, false, 0);

        reg_btn.connect_clicked(move |_btn| {
            // TODO: Attempt to register
        });
        login_btn.connect_clicked(move |_btn| {
            // TODO: Attempt to login
        });

        bm_box.show_all();
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
