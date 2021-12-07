/*
 * Author: Dylan Turner
 * Description: Defines application state
 */

use gtk::{
    Application, ApplicationWindow,
    Box, Orientation,
    main_quit,
    prelude::{ ApplicationExt, WidgetExt, ApplicationExtManual, ContainerExt, BoxExt }
};
use webkit2gtk::{ WebView, traits::WebViewExt };
use serde::{ Serialize, Deserialize };
use log::{ warn };
use log4rs::init_file;
use confy::load;

const GTK_APP_ID: &'static str = "com.blueOkiris.swb";
const WIN_TITLE: &'static str = "Browse the Web";
const WIN_DEF_WIDTH: i32 = 640;
const WIND_DEF_HEIGHT: i32 = 480;
const APP_NAME: &'static str = "swb";

#[derive(Serialize, Deserialize)]
struct AppConfig {
    pub start_page: String,
    pub margin: u32
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            start_page: String::from("https://duckduckgo.org"),
            margin: 10
        }
    }
}

pub struct SimpleBrowser {
    app: Application
}

impl SimpleBrowser {
    // Create the base window given settings
    fn create_window(app: &Application) -> ApplicationWindow {
        let win = ApplicationWindow::builder()
            .application(app)
            .default_width(WIN_DEF_WIDTH).default_height(WIND_DEF_HEIGHT)
            .title(WIN_TITLE)
            .build();
        win.connect_destroy(|_win| { main_quit(); });
        win
    }

    // Create the web view and set start page
    fn create_webview(win: &ApplicationWindow, cfg: &AppConfig) -> WebView {
        let web_view = WebView::builder().build();
        web_view.load_uri(&cfg.start_page);

        let h_box_pad = Box::new(Orientation::Horizontal, 0);
        h_box_pad.pack_start(&web_view, true, true, cfg.margin);
        h_box_pad.show_all();

        let v_box_pad = Box::new(Orientation::Vertical, 0);
        v_box_pad.pack_start(&h_box_pad, true, true, cfg.margin);
        v_box_pad.show_all();
        
        win.add(&v_box_pad);
        web_view
    }

    pub fn new() -> Self {
        // Set up logging
        init_file("logging_config.yaml", Default::default()).unwrap();

        // Set up window
        let app = Application::builder()
            .application_id(GTK_APP_ID)
            .build();
        app.connect_activate(|app| {
            // Load config file
            let cfg = match load(APP_NAME) {
                Err(_) => {
                    warn!("Error in config! Using defaults.");
                    AppConfig::default()
                }, Ok(config) => config
            };

            let win = SimpleBrowser::create_window(&app);
            let web_view = SimpleBrowser::create_webview(&win, &cfg);
            
            win.show_all();
        });

        Self { app }
    }

    pub fn run(&self) {
        self.app.run();
    }
}
