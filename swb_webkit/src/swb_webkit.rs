/*
 * Author: Dylan Turner
 * Description: Browser plugin for swb that uses Webkit Gtk
 */

mod manager;

use webkit2gtk::{
    WebView, traits::{
        WebViewExt
    }
};
use gtk::{
    Box, prelude::BoxExt
};
use crate::manager::{
    WEB_VIEW_MANAGER, WebViewManager
};

const NAME: &'static str = "Swb Webkit";
const START_PAGE: &'static str = "https://duckduckgo.com/";

/* Unused plugin functions */

#[no_mangle]
pub fn on_navbar_load(_navbar: &Box) { }

/* Used plugin functions */

#[no_mangle]
pub fn name() -> String {
    String::from(NAME)
}

// Create the web view in the manager
#[no_mangle]
pub fn on_window_content_load(content: &Box) {
    let web_view = WebView::builder().build();
    web_view.load_uri(START_PAGE);
    content.pack_start(&web_view.clone(), true, true, 0);

    unsafe {
        WEB_VIEW_MANAGER = WebViewManager::new(web_view.clone());
    }
}

#[no_mangle]
pub fn on_back_btn_clicked() {
    unsafe {
        WEB_VIEW_MANAGER.navigate_back();
    }
}

#[no_mangle]
pub fn on_fwd_btn_clicked() {
    unsafe {
        WEB_VIEW_MANAGER.navigate_forward();
    }
}

#[no_mangle]
pub fn on_change_page(url: &String) {
    unsafe {
        WEB_VIEW_MANAGER.navigate(url);
    }
}

#[no_mangle]
pub fn on_refr_btn_clicked() {
    // TODO: Refresh
}
