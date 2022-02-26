/*
 * Author: Dylan Turner
 * Description: Create and launch main window using GTK and set up plugins
 */

use std::{
    env::set_var,
    sync::Arc
};
use gtk4::{
    Application, ApplicationWindow,
    Button, Box, EntryBuffer, Entry,
    Orientation, Align,
    prelude::{
        ApplicationExt, ApplicationExtManual, GtkWindowExt, WidgetExt,
        BoxExt, ButtonExt, EntryExt, EditableExt
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
                .can_focus(true)
                .build();
            win.set_size_request(WIN_MIN_WIDTH, WIN_MIN_HEIGHT);

            Self::create_gui(&win, &plugins);

            win.show();
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
                ..append(&navbar);
        };

        win.set_child(Some(&view));
    }

    // Create the top button bar
    fn create_navbar(plugins: &Vec<Arc<Container<Plugin>>>) -> Box {
        let back_plugins = plugins.clone();
        let back_btn = cascade! {
            Button::with_label("←");
                ..set_margin_end(DEF_MARGIN); // margin to next btn
                ..connect_clicked(move |_btn| {
                    for plugin in &back_plugins {
                        plugin.on_back_btn_clicked();
                    }
                });
        };

        let fwd_plugins = plugins.clone();
        let fwd_btn = cascade! {
            Button::with_label("→");
                ..set_margin_end(DEF_MARGIN); // margin to next btn
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
            Button::with_label("↺");
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
                ..append(&back_btn);
                ..append(&fwd_btn);
                ..append(&search);
                ..append(&refr_btn);
        };

        for plugin in plugins {
            plugin.on_navbar_load(&navbar.clone());
        }

        navbar
    }
}
