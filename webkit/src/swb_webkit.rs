/*
 * Author: Dylan Turner
 * Description: Browser plugin for swb that uses Webkit Gtk and adblock
 */

mod manager;
mod filter;

use webkit2gtk::{
    WebView, WebContext,
    traits::{
        WebViewExt, WebContextExt
    }
};
use gtk::{
    Box, ToggleButton, Button,
    prelude::{
        ContainerExt, BoxExt, ButtonExt, ToggleButtonExt, WidgetExt
    }
};
use dirs::config_dir;
use crate::{
    filter::{
        add_filter, update_filter
    }, manager::{
        WEB_VIEW_MANAGER, WebViewManager
    }
};

const NAME: &'static str = "Swb Webkit";
const START_PAGE: &'static str = "https://duckduckgo.com/";
const DEF_MARGIN: i32 = 5;

/* Unused plugin functions */

#[no_mangle]
pub fn queue_send_msg() -> Option<(String, String)> {
    None
}

/* Used plugin functions */

#[no_mangle]
pub fn name() -> String {
    String::from(NAME)
}

// Check for other plugins redirecting us
#[no_mangle]
pub fn recv_msgs(msgs: &Vec<(String, String)>) {
    for (sndr, msg) in msgs {
        if sndr == &String::from("WEBKIT_REDIRECT") {
            unsafe {
                WEB_VIEW_MANAGER.navigate(msg);
            }
        }
    }
}

// Create the web view in the manager
#[no_mangle]
pub fn on_window_content_load(content: &Box) {
    // Enable adblock
    let web_ctx = WebContext::builder().build();
    let mut conf = config_dir().unwrap();
    conf.push("swb");
    conf.push("webkit");
    web_ctx.set_web_extensions_directory(conf.as_os_str().to_str().unwrap());
 
    // Create the view
    let web_view = WebView::builder().web_context(&web_ctx).build();
    add_filter(&web_view);
    web_view.load_uri(START_PAGE);
    content.pack_start(&web_view.clone(), true, true, 0);

    // Set up global data
    unsafe {
        WEB_VIEW_MANAGER = WebViewManager::new(
            web_view.clone(), content.clone()
        );
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
    unsafe {
        WEB_VIEW_MANAGER.refresh();
    }
}

// Adblock settings manager and private mode
#[no_mangle]
pub fn on_navbar_load(navbar: &Box) {
    let private_btn = create_private_button();
    navbar.add(&private_btn);

    let adblock_refresh_btn = Button::builder()
        .label("⯃").margin_start(DEF_MARGIN)
        .tooltip_text("Update Content Filter")
        .build();
    adblock_refresh_btn.connect_clicked(|_btn| {
        unsafe {
            update_filter(&WEB_VIEW_MANAGER.web_view.clone().unwrap());
        }
    });
    navbar.add(&adblock_refresh_btn);
}

fn create_private_button() -> ToggleButton {
    let private_btn = ToggleButton::builder()
        .label("(¬■_■)").margin_start(DEF_MARGIN)
        .tooltip_text("Toggle Private Browsing")
        .build();
    private_btn.connect_toggled(move |toggle| {
        // Reset with a new web view (REQUIRED by webkit2gtk)

        // Enable adblock
        let web_ctx = WebContext::builder().build();
        let mut conf = config_dir().unwrap();
        conf.push("swb");
        conf.push("webkit");
        web_ctx.set_web_extensions_directory(conf.as_os_str().to_str().unwrap());

        // Create the view
        let web_view = WebView::builder()
            .is_ephemeral(toggle.is_active()).web_context(&web_ctx)
            .build();
        add_filter(&web_view);
        web_view.load_uri(START_PAGE);

        // Get parent and replace old web view with the new one
        let content = unsafe {
            WEB_VIEW_MANAGER.view_parent.clone().unwrap()
        };
        let old_web_view = unsafe {
            WEB_VIEW_MANAGER.web_view.clone().unwrap()
        };
        content.remove(&old_web_view);
        content.pack_start(&web_view.clone(), true, true, 0);
        content.show_all();

        // Set up global data
        unsafe {
            WEB_VIEW_MANAGER = WebViewManager::new(
                web_view.clone(), content.clone()
            );
        }
    });
    private_btn
}

