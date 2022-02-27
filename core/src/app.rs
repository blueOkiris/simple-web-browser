/*
 * Author: Dylan Turner
 * Description: Create and launch main window using GTK and set up plugins
 */

use std::{
    env::set_var,
    sync::Arc
};
use gtk::{
    Application, ApplicationWindow,
    Button, Box, EntryBuffer, Entry,
    Orientation, Align,
    prelude::{
        ApplicationExt, ApplicationExtManual, WidgetExt,
        ButtonExt, EntryExt, ContainerExt
    }
};
use cascade::cascade;
use dlopen::wrapper::Container;
use crate::plugin::{
    load_plugins, Plugin
};

const APP_ID: &'static str = "com.blueokiris.swb";
const GSK_RENDERER: &'static str = "cairo";
const WIN_MIN_WIDTH: i32 = 350;
const WIN_MIN_HEIGHT: i32 = 350;
const WIN_DEF_WIDTH: i32 = 800;
const WIN_DEF_HEIGHT: i32 = 600;
const WIN_TITLE: &'static str = "Simple Web Browser";
const DEF_MARGIN: i32 = 5;
const START_PAGE: &'static str = "https://duckduckgo.com/";

pub struct App {
    plugins: Vec<Arc<Container<Plugin>>>
}

impl App {
    pub fn new() -> Self {
        set_var("GSK_RENDERER", GSK_RENDERER);

        let plugins = load_plugins();

        Self {
            plugins
        }
    }

    pub fn run(self) {
        let gtk_app = Application::builder().application_id(APP_ID).build();
        let plugins = self.plugins.clone();
        gtk_app.connect_activate(move |app| {
            let win = ApplicationWindow::builder()
                .application(app)
                .title(WIN_TITLE)
                .default_width(WIN_DEF_WIDTH).default_height(WIN_DEF_HEIGHT)
                .width_request(WIN_MIN_WIDTH).height_request(WIN_MIN_HEIGHT)
                .can_focus(true)
                .build();

            Self::create_gui(&win, &plugins);

            win.show_all();
        });

        gtk_app.run();
        gtk_app.quit();
    }

    // Set up all the content that goes into the GUI window and its events
    fn create_gui(
            win: &ApplicationWindow,
            plugins: &Vec<Arc<Container<Plugin>>>) {
        let navbar = Self::create_navbar(plugins);
        
        // Main container for everything
        let view = cascade! {
            Box::builder()
                .orientation(Orientation::Vertical)
                .hexpand(true).vexpand(true)
                .margin_top(DEF_MARGIN).margin_bottom(DEF_MARGIN)
                .margin_start(DEF_MARGIN).margin_end(DEF_MARGIN)
                .build();
                ..add(&navbar);
        };

        for plugin in plugins {
            plugin.on_window_content_load(&view.clone());
        }

        win.set_child(Some(&view));
    }

    // Create the top button bar
    fn create_navbar(plugins: &Vec<Arc<Container<Plugin>>>) -> Box {
        let back_plugins = plugins.clone();
        let back_btn = cascade! {
            Button::builder()
                .label("←").margin_end(DEF_MARGIN) // margin to next btn
                .tooltip_text("Navigate Back")
                .build();
                ..connect_clicked(move |_btn| {
                    for plugin in &back_plugins {
                        plugin.on_back_btn_clicked();
                    }
                });
        };

        let fwd_plugins = plugins.clone();
        let fwd_btn = cascade! {
            Button::builder()
                .label("→").margin_end(DEF_MARGIN) // margin to next btn
                .tooltip_text("Navigate Forward")
                .build();
                ..connect_clicked(move |_btn| {
                    for plugin in &fwd_plugins {
                        plugin.on_fwd_btn_clicked();
                    }
                });
        };

        let search_buff = EntryBuffer::new(Some(&String::from(START_PAGE)));
        let search_plugins = plugins.clone();
        let search = cascade! {
            Entry::builder()
                .hexpand(true).valign(Align::Center).buffer(&search_buff)
                .margin_end(DEF_MARGIN)
                .build();
                ..connect_activate(move |entry| {
                    let url = entry.text().to_string().clone();
                    for plugin in &search_plugins {
                        plugin.on_change_page(&url.clone());
                    }
                });
        };

        let refr_plugins = plugins.clone();
        let refr_btn = cascade! {
            Button::builder()
                .label("↺").margin_end(DEF_MARGIN) // margin to next btn
                .tooltip_text("Refresh Page")
                .build();
                ..connect_clicked(move |_btn| {
                    for plugin in &refr_plugins {
                        plugin.on_refr_btn_clicked();
                    }
                });
        };

        let navbar = cascade! {
            Box::builder()
                .orientation(Orientation::Horizontal)
                .margin_bottom(DEF_MARGIN) // margin to web view
                .hexpand(true).vexpand(false).build();
                ..add(&back_btn);
                ..add(&fwd_btn);
                ..add(&search);
                ..add(&refr_btn);
        };

        for plugin in plugins {
            plugin.on_navbar_load(&navbar.clone());
        }

        navbar
    }
}
