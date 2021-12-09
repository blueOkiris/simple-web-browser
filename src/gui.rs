/*
 * Author: Dylan Turner
 * Description: Helper functions for app.rs to contain GUI generation code
 */

use gtk::{
    Box, EntryBuffer, Entry, Label, Dialog, ToggleButton, Window,
    Button,
    Orientation, ResponseType, DialogFlags,
    prelude::{ BoxExt, DialogExt, GtkWindowExt, WidgetExt, ContainerExt }
};
use log::{ error };
use cascade::cascade;

const POPUP_WIDTH: i32 = 300;
const POPUP_HEIGHT: i32 = 200;

pub struct LoginState {
    pub dialog: Dialog,
    pub uname_buff: EntryBuffer,
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
        ..set_default_size(POPUP_WIDTH, POPUP_HEIGHT);
        ..set_modal(true);
        ..set_resizable(false);
        ..add_button("Register", ResponseType::Apply);
        ..add_button("Login", ResponseType::Accept);
        ..add_button("Cancel", ResponseType::Cancel);
    };
    let content_area = dialog.content_area();

    let (uname, uname_buff) = create_username_box(padding);
    content_area.pack_start(&uname, true, true, padding);
    
    let (pword, pword_buff) = create_password_box(padding);
    content_area.pack_start(&pword, true, true, padding);

    let remem_box = ToggleButton::with_label("Remember");
    let remem = cascade! {
        Box::new(Orientation::Horizontal, 0);
            ..pack_start(&remem_box, true, true, padding);
    };
    content_area.pack_start(&remem, true, true, padding);

    LoginState { dialog, uname_buff, pword_buff, remem_box }
}

// Generate a field for a username textbox
fn create_username_box(padding: u32) -> (Box, EntryBuffer) {
    let uname_buff = EntryBuffer::new(Some(""));
    let uname = cascade! {
        Box::new(Orientation::Horizontal, 0);
            ..pack_start(
                &Label::new(Some("Username: ")), false, false, padding
            );..pack_start(
                &Entry::builder().buffer(&uname_buff).hexpand(true).build(),
                true, true, padding
            );
    };
    (uname, uname_buff)
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
            ..connect_response(move |view, _| { view.hide(); });
            ..set_modal(true);
    };
    let _con = cascade! {
        err_dialog.content_area();
            ..pack_start(
                &Label::new(Some(format!("{}", err).as_str())),
                true, true, 0
            );
    };
    err_dialog.show_all();
}
