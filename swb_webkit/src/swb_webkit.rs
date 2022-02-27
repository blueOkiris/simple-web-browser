/*
 * Author: Dylan Turner
 * Description: Browser plugin for swb that uses Webkit Gtk
 */

mod manager;

use webkit2gtk::{
    WebView, WebContext,
    traits::{
        WebViewExt, WebContextExt
    }
};
use gtk::{
    Box, Button,
    prelude::{
        ContainerExt, BoxExt, ButtonExt
    }
};
use crate::manager::{
    WEB_VIEW_MANAGER, WebViewManager
};

const NAME: &'static str = "Swb Webkit";
const START_PAGE: &'static str = "https://duckduckgo.com/";
const DEF_MARGIN: i32 = 5;
const ADBLOCK_EXTENSION_DIR: &'static str = "adblock";

/* Used plugin functions */

#[no_mangle]
pub fn name() -> String {
    String::from(NAME)
}

// Create the web view in the manager
#[no_mangle]
pub fn on_window_content_load(content: &Box) {
    // Enable adblock
    let web_ctx = WebContext::builder().build();
    web_ctx.set_web_extensions_directory(ADBLOCK_EXTENSION_DIR);

    // Create the view
    let web_view = WebView::builder().web_context(&web_ctx).build();
    web_view.load_uri(START_PAGE);
    content.pack_start(&web_view.clone(), true, true, 0);

    // Set up global data
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

// Adblock settings manager
#[no_mangle]
pub fn on_navbar_load(navbar: &Box) {
    let settings_mgr = create_settings_manager();
    navbar.add(&settings_mgr);
}

fn create_settings_manager() -> Button {
    // Create a mini web view in our menu to manage blockit
    let web_ctx = WebContext::builder().build(); // Add adblock here as well
    web_ctx.set_web_extensions_directory(ADBLOCK_EXTENSION_DIR);
    let viewer = WebView::builder().web_context(&web_ctx).build();

    // Don't draw or anything as it starts up in a separate window

    let adblock_starter = Button::builder()
        .label("⯃").margin_start(DEF_MARGIN)
        .build();
    adblock_starter.connect_clicked(move |_btn| {
        viewer.load_uri("blockit://settings");
    });
    
    adblock_starter
}
