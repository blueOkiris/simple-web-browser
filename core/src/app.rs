/*
 * Author: Dylan Turner
 * Description: Create and launch main window using GTK and set up plugins
 */

use std::{
    thread::spawn,
    env::set_var,
    sync::{
        Arc, Mutex,
        mpsc::{ Sender, Receiver, channel }
    }
};
use gtk4::{
    Application, ApplicationWindow,
    prelude::{ ApplicationExt, ApplicationExtManual, WidgetExt }
};
use crate::{
    event::{ AsyncEvent, AsyncGuiInfo },
    plugin::Plugin
};

const APP_ID: &'static str = "com.blueokiris.swb";
const GSK_RENDERER: &'static str = "cairo";
const WIN_MIN_WIDTH: i32 = 300;
const WIN_MIN_HEIGHT: i32 = 300;
const WIN_DEF_WIDTH: i32 = 800;
const WIN_DEF_HEIGHT: i32 = 600;
const WIN_TITLE: &'static str = "Simple Web Browser";

pub struct App {
    event_tx: Sender<AsyncEvent>,
    event_rx: Receiver<AsyncEvent>,
    gui_tx: Sender<AsyncGuiInfo>,
    gui_rx: Arc<Mutex<Receiver<AsyncGuiInfo>>>,
    plugins: Vec<Plugin>
}

impl App {
    pub fn new() -> Self {
        set_var("GSK_RENDERER", GSK_RENDERER);

        let (event_tx, event_rx) = channel();
        let (gui_tx, gui_rx) = channel();

        let plugins = Plugin::from_folder();

        Self {
            event_tx, event_rx,
            gui_tx, gui_rx: Arc::new(Mutex::new(gui_rx)),
            plugins
        }
    }

    pub fn run(self) {
        let gtk_app = Application::builder().application_id(APP_ID).build();
        let _event_tx = self.event_tx.clone();
        let _gui_rx = self.gui_rx.clone();
        gtk_app.connect_activate(move |app| {
            let win = ApplicationWindow::builder()
                .application(app)
                .title(WIN_TITLE)
                .default_width(WIN_DEF_WIDTH).default_height(WIN_DEF_HEIGHT)
                .can_focus(true)
                .build();
            win.set_size_request(WIN_MIN_WIDTH, WIN_MIN_HEIGHT);

            win.show();
        });

        // Send events from Gtk to me here
        let _gui_tx = self.gui_tx.clone();
        spawn(move || {
            while let Ok(event) = self.event_rx.recv() {
                match event.ev_type {
                    
                }
            }
        });

        // TODO: Remove this when we have more than just structured code
        for plugin in self.plugins {
            println!("Using plugin {}", plugin.call_name());
        }

        gtk_app.run();
        gtk_app.quit();
    }
}
