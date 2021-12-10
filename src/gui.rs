/*
 * Author: Dylan Turner
 * Description: Helper functions for app.rs to contain GUI generation code
 */

use gtk::{
    Box, EntryBuffer, Entry, Label, Dialog, ToggleButton, Window,
    Orientation, ResponseType, DialogFlags, ScrolledWindow, Adjustment,
    prelude::{ BoxExt, DialogExt, GtkWindowExt, WidgetExt, ContainerExt }
};
use log::{ error, info };
use cascade::cascade;

const LOGIN_POPUP_WIDTH: i32 = 300;
const LOGIN_POPUP_HEIGHT: i32 = 200;

const BM_POPUP_WIDTH: i32 = 350;
const BM_POPUP_HEIGHT: i32 = 80;

pub struct LoginState {
    pub dialog: Dialog,
    pub email_buff: EntryBuffer,
    pub pword_buff: EntryBuffer,
    pub remem_box: ToggleButton
}

// Create the login popup
pub fn create_login_dialog(padding: u32, win: &Window) -> LoginState {
    let dialog = cascade! {
        Dialog::with_buttons(
            Some("Sign In"), Some(win),
            DialogFlags::from_bits(1).unwrap(),
            &[ ]
        );
        ..set_default_size(LOGIN_POPUP_WIDTH, LOGIN_POPUP_HEIGHT);
        ..set_modal(true);
        ..set_resizable(false);
        ..add_button("Register", ResponseType::Apply);
        ..add_button("Login", ResponseType::Accept);
        ..add_button("Cancel", ResponseType::Cancel);
    };
    let content_area = dialog.content_area();

    let (email, email_buff) = create_email_box(padding);
    content_area.pack_start(&email, true, true, padding);
    
    let (pword, pword_buff) = create_password_box(padding);
    content_area.pack_start(&pword, true, true, padding);

    let remem_box = ToggleButton::with_label("Click to Remember");
    let remem = cascade! {
        Box::new(Orientation::Horizontal, 0);
            ..pack_start(&remem_box, true, true, padding);
    };
    content_area.pack_start(&remem, true, true, padding);

    LoginState { dialog, email_buff, pword_buff, remem_box }
}

// Generate a field for a username textbox
fn create_email_box(padding: u32) -> (Box, EntryBuffer) {
    let email_buff = EntryBuffer::new(Some(""));
    let email = cascade! {
        Box::new(Orientation::Horizontal, 0);
            ..pack_start(
                &Label::new(Some("Username: ")), false, false, padding
            );..pack_start(
                &Entry::builder().buffer(&email_buff).hexpand(true).build(),
                true, true, padding
            );
    };
    (email, email_buff)
}

// Do the same with a password
fn create_password_box(padding: u32) -> (Box, EntryBuffer) {
    let pword_buff = EntryBuffer::new(Some(""));
    let pword = cascade! {
        Box::new(Orientation::Horizontal, 0);
            ..pack_start(
                &Label::new(Some("Password: ")), false, false, padding
            );..pack_start(
                &Entry::builder()
                    .buffer(&pword_buff).hexpand(true).visibility(false)
                    .build(),
                true, true, padding
            );
    };
    (pword, pword_buff)
}

// Generate a popup window that displays and error message
pub fn create_error_popup(err: &String) {
    error!("Error logging in {}.", err);
    let err_dialog = cascade! {
        Dialog::new();
            ..set_title("Error!");
            ..add_button("Okay", ResponseType::Cancel);
            ..set_default_size(LOGIN_POPUP_WIDTH, LOGIN_POPUP_HEIGHT);
            ..connect_response(move |view, _| { view.hide(); });
            ..set_modal(true);
    };
    let _con = cascade! {
        err_dialog.content_area();
            ..pack_start(&create_scrollable_label(&err), true, true, 0);
    };
    err_dialog.show_all();
}

pub fn create_scrollable_label(msg: &String) -> ScrolledWindow {
    let scroll_view = ScrolledWindow::new::<Adjustment, Adjustment>(None, None);
    scroll_view.add(&Label::new(Some(msg.as_str())));
    scroll_view
}

pub fn create_success_popup(msg: &String) {
    info!("{}.", msg);
    let success_dialog = cascade! {
        Dialog::new();
            ..set_title("Success!");
            ..add_button("Okay", ResponseType::Cancel);
            ..connect_response(move |view, _| { view.hide(); });
            ..set_modal(true);
    };
    let _con = cascade! {
        success_dialog.content_area();
            ..pack_start(
                &Label::new(Some(format!("{}", msg).as_str())),
                true, true, 0
            );
    };
    success_dialog.show_all();
}

pub struct BookmarkResult {
    pub dialog: Dialog,
    pub name: EntryBuffer,
    pub fldr: EntryBuffer,
    pub url: String
}

// Create popup for adding a new bookmark
pub fn create_bookmark_add_dialog(
        padding: u32, win: &Window, url: &String) -> BookmarkResult {
    let dialog = cascade! {
        Dialog::with_buttons(
            Some("Add Bookmark"), Some(win),
            DialogFlags::from_bits(1).unwrap(),
            &[ ]
        );
        ..set_default_size(BM_POPUP_WIDTH, BM_POPUP_HEIGHT);
        ..set_modal(true);
        ..set_resizable(false);
        ..add_button("Add", ResponseType::Accept);
        ..add_button("Cancel", ResponseType::Cancel);
    };
    let content_area = dialog.content_area();

    let (name, name_buff) = create_name_field(padding);
    content_area.pack_start(&name, true, true, padding);

    let (fldr, fldr_buff) = create_fldr_field(padding);
    content_area.pack_start(&fldr, true, true, padding);

    content_area.pack_start(
        &create_scrollable_label(&url), true, true, padding
    );

    BookmarkResult {
        dialog,
        name: name_buff,
        fldr: fldr_buff,
        url: url.clone()
    }
}

// Generate a field for a name textbox
fn create_name_field(padding: u32) -> (Box, EntryBuffer) {
    let name_buff = EntryBuffer::new(Some(""));
    let name = cascade! {
        Box::new(Orientation::Horizontal, 0);
            ..pack_start(
                &Label::new(Some("Name: ")), false, false, padding
            );..pack_start(
                &Entry::builder().buffer(&name_buff).hexpand(true).build(),
                true, true, padding
            );
    };
    (name, name_buff)
}

fn create_fldr_field(padding: u32) -> (Box, EntryBuffer) {
    let fldr_buff = EntryBuffer::new(Some(""));
    let fldr = cascade! {
        Box::new(Orientation::Horizontal, 0);
            ..pack_start(
                &Label::new(Some("Folder (Blank is TopLevel): ")),
                false, false, padding
            );..pack_start(
                &Entry::builder().buffer(&fldr_buff).hexpand(true).build(),
                true, true, padding
            );
    };
    (fldr, fldr_buff)
}

// Create popup for adding a new folder
pub fn create_folder_add_dialog(
        padding: u32, win: &Window, url: &String) -> BookmarkResult {
    let dialog = cascade! {
        Dialog::with_buttons(
            Some("Add Folder"), Some(win),
            DialogFlags::from_bits(1).unwrap(),
            &[ ]
        );
        ..set_default_size(BM_POPUP_WIDTH, BM_POPUP_HEIGHT);
        ..set_modal(true);
        ..set_resizable(false);
        ..add_button("Add", ResponseType::Accept);
        ..add_button("Cancel", ResponseType::Cancel);
    };
    let content_area = dialog.content_area();

    let (name, name_buff) = create_name_field(padding);
    content_area.pack_start(&name, true, true, padding);

    content_area.pack_start(
        &create_scrollable_label(&url), true, true, padding
    );

    BookmarkResult {
        dialog,
        name: name_buff.clone(),
        fldr: name_buff,
        url: url.clone()
    }
}

// Create popup for deleting a new bookmark
pub fn create_bookmark_delete_dialog(
        padding: u32, win: &Window) -> BookmarkResult {
    let dialog = cascade! {
        Dialog::with_buttons(
            Some("Delete Bookmark"), Some(win),
            DialogFlags::from_bits(1).unwrap(),
            &[ ]
        );
        ..set_default_size(BM_POPUP_WIDTH, BM_POPUP_HEIGHT);
        ..set_modal(true);
        ..set_resizable(false);
        ..add_button("Delete", ResponseType::Accept);
        ..add_button("Cancel", ResponseType::Cancel);
    };
    let content_area = dialog.content_area();

    let (name, name_buff) = create_name_field(padding);
    content_area.pack_start(&name, true, true, padding);

    BookmarkResult {
        dialog,
        name: name_buff.clone(),
        fldr: name_buff,
        url: String::new()
    }
}
