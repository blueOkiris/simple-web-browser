/*
 * Author: Dylan Turner
 * Description: Plugin for swb that adds bookmark functionality
 */

mod sync;
mod config;

use gtk::{
    MenuButton, Box, Popover, ScrolledWindow, Frame, MenuItem, MenuBar,
    Entry, Label, CheckButton, Button, Dialog, TextView, TextBuffer,
    Orientation, ArrowType, Align, ResponseType, PackDirection,
    prelude::{
        ContainerExt, ButtonExt, WidgetExt, BoxExt, EntryExt, DialogExt,
        GtkWindowExt, MenuButtonExt, ToggleButtonExt, GtkMenuItemExt
    }
};
use cascade::cascade;
use crate::{
    config::Config,
    sync::{
        login, register, get_bookmarks,
        request_pword_reset, reset_password
    }
};

const NAME: &'static str = "Swb Bookmarks";
const DEF_MARGIN: i32 = 5;
const POPOVER_WIDTH: i32 = 400;
const SYNC_POPOVER_HEIGHT: i32 = 285;
const BM_POPOVER_HEIGHT: i32 = 400;
const POPUP_WIDTH: i32 = 250;
const POPUP_HEIGHT: i32 = 100;

pub static mut MSG_QUEUE: Option<Vec<(String, String)>> = None;

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
#[no_mangle]
pub fn recv_msgs(_msgs: &Vec<(String, String)>) { }

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

#[no_mangle]
pub fn queue_send_msg() -> Option<(String, String)> {
    unsafe {
        if MSG_QUEUE.clone().is_none() || MSG_QUEUE.clone().unwrap().len() < 1 {
            None
        } else {
            let mut msg_queue = MSG_QUEUE.clone().unwrap();
            let msg = msg_queue.pop();
            MSG_QUEUE = Some(msg_queue);
            msg
        }
    }
}

// Create menu for logging and syncing passwords and bookmarks
fn create_sync_menu() -> MenuButton {
    // Container for data
    let popover_content = Box::builder()
        .orientation(Orientation::Vertical)
        .margin_bottom(DEF_MARGIN).margin_top(DEF_MARGIN)
        .margin_start(DEF_MARGIN).margin_end(DEF_MARGIN)
        .build();
    let popover = Popover::builder()
        .width_request(POPOVER_WIDTH).height_request(SYNC_POPOVER_HEIGHT)
        .child(&popover_content)
        .build();

    // Sync gets managed in on click later
    let sync_box = Box::builder()
        .hexpand(true).vexpand(true).orientation(Orientation::Vertical)
        .build();
    let sync_scroller = ScrolledWindow::builder()
        .hexpand(true).vexpand(true)
        .child(&sync_box)
        .build();
    let sync_frame = Frame::builder()
        .label("Sync:").hexpand(true).vexpand(true)
        .child(&sync_scroller)
        .build();
        popover_content.add(&sync_frame);

    popover.show_all();
    popover.hide();

    let sync_menu = MenuButton::builder()
        .label("⇅").margin_start(DEF_MARGIN)
        .hexpand(false).direction(ArrowType::Down)
        .tooltip_text("Sync Menu")
        .popover(&popover)
        .build();
    let sync_events = move |menu_btn: &MenuButton| {
        // Reset the view
        for child in sync_box.children().clone() {
            sync_box.remove(&child);
        }

        // Could clean up to reuse code, but it's not too bad rn
        let mut cfg = Config::get_global();
        if cfg.logged_in || cfg.stay_logged_in {
            if !cfg.logged_in {
                // Try to log in
                let email = cfg.clone().email;
                let pword = cfg.clone().pword;
                let log_res = login(&email, &pword);
                cfg.logged_in = log_res.is_ok();
                Config::set_global(cfg.clone());
            }

            if cfg.logged_in {
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

                sync_box.pack_start(&log_out_btn, true, true, 0);
                sync_box.show_all();

                return;
            }
        }

        let email_hbox = Box::builder()
            .orientation(Orientation::Horizontal)
            .hexpand(true).margin_bottom(DEF_MARGIN)
            .margin_start(DEF_MARGIN).margin_end(DEF_MARGIN)
            .build();
        let email_label = Label::builder()
            .label("Email:        ")
            .margin_end(DEF_MARGIN).halign(Align::Start)
            .build();
        let email = Entry::builder().hexpand(true).build();
        email_hbox.pack_start(&email_label, false, false, 0);
        email_hbox.pack_start(&email, true, true, 0);
        sync_box.pack_start(&email_hbox, false, false, 0);

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
        sync_box.pack_start(&pword_hbox, false, false, 0);

        let remember = CheckButton::builder()
            .label("Stay logged in").margin_bottom(DEF_MARGIN)
            .hexpand(true).halign(Align::Center)
            .build();
        sync_box.pack_start(&remember, false, false, 0);

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
        sync_box.pack_start(&btn_box, false, false, 0);

        let reset_req_btn = Button::builder()
            .label("Request Password Change")
            .margin_bottom(DEF_MARGIN).hexpand(true).halign(Align::Center).build();
        sync_box.pack_start(&reset_req_btn, false, false, 0);

        let reset_code_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .hexpand(true).margin_bottom(DEF_MARGIN)
            .margin_start(DEF_MARGIN).margin_end(DEF_MARGIN)
            .build();
        let reset_code_label = Label::builder()
            .label("Reset Code:").margin_end(DEF_MARGIN).halign(Align::Start)
            .build();
        let reset_code = Entry::builder().hexpand(true).visibility(false).build();
        reset_code_box.pack_start(&reset_code_label, false, false, 0);
        reset_code_box.pack_start(&reset_code, true, true, 0);
        sync_box.pack_start(&reset_code_box, false, false, 0);
        let reset_btn = Button::builder()
            .margin_start(DEF_MARGIN).margin_end(DEF_MARGIN)
            .label("Reset Password").build();
        sync_box.pack_start(&reset_btn, false, false, 0);

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
                                    .text("Successfully registered user.")
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
        let log_popover_clone = popover.clone();
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
                    // Show message confirming login
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
                                    .text("Successfully logged user in.")
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

                    log_popover_clone.hide();
                }
            }
        });

        let reset_req_email = email.clone();
        reset_req_btn.connect_clicked(move |_btn| {
            let email_txt = reset_req_email.text().to_ascii_lowercase();
            let reset_res = request_pword_reset(&email_txt);
            match reset_res {
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
                                    .text("Successfully sent reset request.")
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
                }
            }
        });

        let reset_email = email.clone();
        let reset_pword = pword.clone();
        let code = reset_code.clone();
        reset_btn.connect_clicked(move |_btn| {
            let email_txt = reset_email.text().to_ascii_lowercase();
            let pword_txt = reset_pword.text();
            let code_txt = code.text();
            let reset_res = reset_password(&email_txt, &pword_txt, &code_txt);
            match reset_res {
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
                                    .text("Successfully reset password.")
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
                }
            }
        });

        sync_box.show_all();
    };
    sync_menu.connect_clicked(sync_events.clone()); // Login also when clicked
    sync_events(&sync_menu.clone());

    sync_menu
}

// Create menu for managing bookmarks
fn create_bm_menu() -> MenuButton {
    // Container for data
    let popover_content = Box::builder()
        .orientation(Orientation::Vertical)
        .margin_bottom(DEF_MARGIN).margin_top(DEF_MARGIN)
        .margin_start(DEF_MARGIN).margin_end(DEF_MARGIN)
        .build();
    let popover = Popover::builder()
        .width_request(POPOVER_WIDTH).height_request(BM_POPOVER_HEIGHT)
        .child(&popover_content)
        .build();

    // Later these get bookmarks filled in
    let bm_menu = MenuBar::builder()
        .hexpand(true).vexpand(true)
        .pack_direction(PackDirection::Ttb)
        .build();
    let bm_scroller = ScrolledWindow::builder()
        .hexpand(true).vexpand(true)
        .child(&bm_menu)
        .build();
    let bm_frame = Frame::builder()
        .label("Bookmarks:").hexpand(true).vexpand(true)
        .child(&bm_scroller)
        .build();
    popover_content.add(&bm_frame);

    // Add control system
    let name_hbox = Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true).margin_bottom(DEF_MARGIN)
        .margin_start(DEF_MARGIN).margin_end(DEF_MARGIN).margin_top(DEF_MARGIN)
        .build();
    let name_label = Label::builder()
        .label("Path:")
        .margin_end(DEF_MARGIN).halign(Align::Start)
        .build();
    let name = Entry::builder().hexpand(true).build();
    name_hbox.pack_start(&name_label, false, false, 0);
    name_hbox.pack_start(&name, true, true, 0);
    popover_content.pack_start(&name_hbox, false, false, 0);

    let bm_hbox = Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true).margin_bottom(DEF_MARGIN)
        .margin_start(DEF_MARGIN).margin_end(DEF_MARGIN)
        .build();
    let bm_label = Label::builder()
        .label("Bookmark Url:")
        .margin_end(DEF_MARGIN).halign(Align::Start)
        .build();
    let url = Entry::builder().hexpand(true).margin_end(DEF_MARGIN).build();
    let set_btn = cascade! {
        Button::builder() // Can't use with_label here: crashes w/ gtk::init()
            .label("Set")
            .margin_top(DEF_MARGIN).margin_bottom(DEF_MARGIN)
            .build();
            ..connect_clicked(move |_btn| {
                // TODO: Add bookmark or edit
            });
    };
    bm_hbox.pack_start(&bm_label, false, false, 0);
    bm_hbox.pack_start(&url, true, true, 0);
    bm_hbox.pack_start(&set_btn, false, false, 0);
    popover_content.pack_start(&bm_hbox, false, false, 0);

    let add_fldr_btn = cascade! {
        Button::builder() // Can't use with_label here: crashes w/ gtk::init()
            .label("Add Folder At Path")
            .margin_bottom(DEF_MARGIN)
            .build();
            ..connect_clicked(move |_btn| {
                // TODO: Add folder
            });
    };
    popover_content.add(&add_fldr_btn);

    let rm_btn = cascade! {
        Button::builder() // Can't use with_label here: crashes w/ gtk::init()
            .label("Remove Item At Path")
            .build();
            ..connect_clicked(move |_btn| {
                // TODO: Remove bookmark or folder

            });
    };
    popover_content.add(&rm_btn);

    popover.show_all();
    popover.hide();

    let bm_btn = MenuButton::builder()
        .label("🕮").margin_start(DEF_MARGIN)
        .direction(ArrowType::Down).popover(&popover)
        .tooltip_text("Bookmarks Menu")
        .build();
    let load_bms = move |_menu_btn: &MenuButton| {
        // Reset the view
        for child in bm_menu.children().clone() {
            bm_menu.remove(&child);
        }

        // Try to sync
        let bm_col = match get_bookmarks() {
            Err(err) => {
                println!("Failed to sync bookmarks: {} Continuing local", err);
                let cfg = Config::get_global();
                cfg.bm_collection
            }, Ok(bm_collection) => {
                let mut cfg = Config::get_global();
                cfg.bm_collection = bm_collection.clone();
                Config::set_global(cfg);
                Config::store_global();

                bm_collection
            }
        };

        // Add folders first
        let subfldr_items = bm_col.get_subfldr_menu_item();
        for fldr_item in subfldr_items.iter() {
            bm_menu.add(fldr_item);
        }

        // Now add the regular bookmarks
        for bookmark in bm_col.bms {
            let bm = MenuItem::builder()
                .label(&bookmark.0)
                .hexpand(true).vexpand(false)
                .build();
            let url_clone = bookmark.1.clone();
            bm.connect_activate(move |_menu_item| {
                unsafe {
                    if MSG_QUEUE.clone().is_none() {
                        MSG_QUEUE = Some(Vec::new())
                    }
                    let mut queue = MSG_QUEUE.clone().unwrap();
                    queue.push((
                        String::from("WEBKIT_REDIRECT"),
                        url_clone.clone()
                    ));
                    MSG_QUEUE = Some(queue);
                }
            });

            bm_menu.add(&bm);
        }

        bm_menu.show_all();
    };

    // Idk why I couldn't load it as a param like w/ sync button, but oh well
    bm_btn.connect_clicked(load_bms.clone());

    bm_btn
}
