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
    Box, ToggleButton, Button,
    prelude::{
        ContainerExt, BoxExt, ButtonExt, ToggleButtonExt, WidgetExt
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

    let settings_mgr = create_settings_manager();
    navbar.add(&settings_mgr);
}

fn create_private_button() -> ToggleButton {
    let private_btn = ToggleButton::builder()
        .label("(¬■_■)").margin_start(DEF_MARGIN)
        .build();
    private_btn.connect_toggled(move |toggle| {
        // Reset with a new web view (REQUIRED by webkit2gtk)

        // Enable adblock
        let web_ctx = WebContext::builder().build();
        web_ctx.set_web_extensions_directory(ADBLOCK_EXTENSION_DIR);

        // Create the view
        let web_view = WebView::builder()
            .is_ephemeral(toggle.is_active()).web_context(&web_ctx)
            .build();
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
