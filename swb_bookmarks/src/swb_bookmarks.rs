/*
 * Author: Dylan Turner
 * Description: Browser plugin for swb that uses Webkit Gtk
 */

mod sync;
mod config;

use gtk::{
    MenuButton, Box, Popover, ScrolledWindow, Frame,
    Entry, Label, CheckButton, Button, Dialog, TextView, TextBuffer,
    Orientation, ArrowType, Align, ResponseType,
    prelude::{
        ContainerExt, ButtonExt, WidgetExt, BoxExt, EntryExt, DialogExt,
        GtkWindowExt, MenuButtonExt
    }, traits::ToggleButtonExt 
};
use cascade::cascade;
use crate::{
    config::Config,
    sync::{
        login, register
    }
};

const NAME: &'static str = "Swb Bookmarks";
const DEF_MARGIN: i32 = 5;
const POPOVER_WIDTH: i32 = 400;
const POPOVER_HEIGHT: i32 = 200;
const POPUP_WIDTH: i32 = 250;
const POPUP_HEIGHT: i32 = 100;

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

        // Could clean up to reuse code, but it's not too bad rn
        let cfg = Config::get_global();
        if cfg.logged_in || cfg.stay_logged_in {
            let mut logged_in = cfg.logged_in;
            if !logged_in {
                // Try to log in
                let email = cfg.email;
                let pword = cfg.pword;
                let log_res = login(&email, &pword);
                logged_in = log_res.is_ok()
            }

            if logged_in {
                let log_out_btn = Button::builder()
                    .label("Sign Out")
                    .margin_bottom(DEF_MARGIN).margin_top(DEF_MARGIN)
                    .margin_start(DEF_MARGIN).margin_end(DEF_MARGIN)
                    .hexpand(true).vexpand(true)
                    .build();

                let menu_btn_clone = menu_btn.clone();
                log_out_btn.connect_clicked(move |_btn| {
                    let mut cfg = Config::get_global();
                    cfg.logged_in = false;
                    cfg.stay_logged_in = false;
                    Config::set_global(cfg);
                    // Don't store here as this is temp sign out
                    menu_btn_clone.popover().unwrap().hide();
                });

                bm_box.pack_start(&log_out_btn, true, true, 0);
                bm_box.show_all();

                return;
            }
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
            .label("Stay logged in").margin_bottom(DEF_MARGIN)
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

        // Handle login/registration
        let reg_email = email.clone();
        let reg_pword = pword.clone();
        let reg_remem = remember.clone();
        reg_btn.connect_clicked(move |_btn| {
            let email_txt = reg_email.text().to_ascii_lowercase();
            let pword_txt = reg_pword.text().to_string();
            let reg_res = register(&email_txt, &pword_txt);
            match reg_res {
                Err(err) => {
                    let dialog = cascade! {
                        Dialog::builder()
                            .title("Error")
                            .width_request(POPUP_WIDTH)
                            .height_request(POPUP_HEIGHT)
                            .build();
                            ..add_button("Close", ResponseType::Apply);
                            ..set_modal(false);
                            ..set_resizable(false);
                    };
                    dialog.content_area().pack_start(
                        &TextView::builder()
                            .editable(false)
                            .buffer(
                                &TextBuffer::builder()
                                    .text(&err.to_string())
                                    .build()
                            ).hexpand(true).vexpand(true).can_focus(false)
                            .build(),
                        true, true, DEF_MARGIN as u32
                    );
                    dialog.connect_response(|mini_win, resp| {
                        if resp == ResponseType::Apply {
                            mini_win.hide()
                        }
                    });
                    dialog.show_all();
                }, Ok(()) => {
                    // Show message confirming registration
                    let dialog = cascade! {
                        Dialog::builder()
                            .title("Success")
                            .width_request(POPUP_WIDTH)
                            .height_request(POPUP_HEIGHT)
                            .build();
                            ..add_button("Close", ResponseType::Apply);
                            ..set_modal(false);
                            ..set_resizable(false);
                    };
                    dialog.content_area().pack_start(
                        &TextView::builder()
                            .editable(false)
                            .buffer(
                                &TextBuffer::builder()
                                    .text("Succesfully registered user")
                                    .build()
                            ).hexpand(true).vexpand(true).can_focus(false)
                            .build(),
                        true, true, DEF_MARGIN as u32
                    );
                    dialog.connect_response(move |mini_win, resp| {
                        if resp == ResponseType::Apply {
                            mini_win.hide();
                        }
                    });
                    dialog.show_all();

                    // Save hashed password locally in config
                    if reg_remem.is_active() {
                        let mut cfg = Config::get_global();
                        cfg.stay_logged_in = true;
                        cfg.email = email_txt.clone();
                        cfg.pword = pword_txt.clone();
                        Config::set_global(cfg);
                        Config::store_global();
                    }
                }
            }
        });
        let log_email = email.clone();
        let log_pword = pword.clone();
        let log_remem = remember.clone();
        let log_menu_clone = menu.clone();
        login_btn.connect_clicked(move |_btn| {
            let email_txt = log_email.text().to_ascii_lowercase();
            let pword_txt = log_pword.text().to_string();
            let log_res = login(&email_txt, &pword_txt);
            match log_res {
                Err(err) => {
                    let dialog = cascade! {
                        Dialog::builder()
                            .title("Error")
                            .width_request(POPUP_WIDTH)
                            .height_request(POPUP_HEIGHT)
                            .build();
                            ..add_button("Close", ResponseType::Apply);
                            ..set_modal(false);
                            ..set_resizable(false);
                    };
                    dialog.content_area().pack_start(
                        &TextView::builder()
                            .editable(false)
                            .buffer(
                                &TextBuffer::builder()
                                    .text(&err.to_string())
                                    .build()
                            ).hexpand(true).vexpand(true).can_focus(false)
                            .build(),
                        true, true, DEF_MARGIN as u32
                    );
                    dialog.connect_response(|mini_win, resp| {
                        if resp == ResponseType::Apply {
                            mini_win.hide()
                        }
                    });
                    dialog.show_all();
                }, Ok(()) => {
                    // Show message confirming registration
                    let dialog = cascade! {
                        Dialog::builder()
                            .title("Success")
                            .width_request(POPUP_WIDTH)
                            .height_request(POPUP_HEIGHT)
                            .build();
                            ..add_button("Close", ResponseType::Apply);
                            ..set_modal(false);
                            ..set_resizable(false);
                    };
                    dialog.content_area().pack_start(
                        &TextView::builder()
                            .editable(false)
                            .buffer(
                                &TextBuffer::builder()
                                    .text("Succesfully logged user in")
                                    .build()
                            ).hexpand(true).vexpand(true).can_focus(false)
                            .build(),
                        true, true, DEF_MARGIN as u32
                    );
                    dialog.connect_response(move |mini_win, resp| {
                        if resp == ResponseType::Apply {
                            mini_win.hide();
                        }
                    });
                    dialog.show_all();

                    let mut cfg = Config::get_global();

                    // Save hashed password locally in config
                    if log_remem.is_active() {
                        cfg.stay_logged_in = true;
                        cfg.email = email_txt.clone();
                        cfg.pword = pword_txt.clone();
                    }

                    cfg.logged_in = true;
                    Config::set_global(cfg);
                    Config::store_global();

                    log_menu_clone.hide();
                }
            }
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
