/*
 * Author: Dylan Turner
 * Description: Defines application state
 */

use std::future::Future;
use async_channel::{ unbounded, Sender };
use gtk::{
    main_quit, Inhibit, init, main,
    Button, Box, Orientation, TextView, Grid, TextBuffer,
    Window, WindowType,
    prelude::{
        ContainerExt, ButtonExt, BoxExt, WidgetExt, GtkWindowExt, GridExt,
        TextBufferExt
    }, glib::{ set_program_name, set_application_name, MainContext }
};
use webkit2gtk::{ WebView, LoadEvent, traits::WebViewExt };
use serde::{ Serialize, Deserialize };
use log::{ warn, error, info };
use confy::load;
use cascade::cascade;

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

// Spawns a task on default executor, without waiting to complete
fn spawn<F>(future: F) where F: Future<Output = ()> + 'static {
    MainContext::default().spawn_local(future);
}

pub fn start_browser() {
    set_program_name(APP_NAME.into());
    set_application_name(APP_NAME);

    // Initialize gtk
    if init().is_err() {
        error!("Failed to initialize GTK Application!");
        panic!("Failed to initialize GTK Application!");
    }

    // Attach tx to widgets and rx to handler
    let (tx, rx) = unbounded();
    let mut app = AppState::new(tx);
    let mut via_nav_btns = false;
    let event_handler = async move {
        while let Ok(event) = rx.recv().await {
            match event.tp {
                EventType::BackClicked => {
                    if app.url_index > 0 {
                        app.url_index -= 1;
                        app.web_view.load_uri(
                            app.visited_urls[app.url_index].as_str()
                        );
                        via_nav_btns = true;

                        info!(
                            "Back to {}.", app.visited_urls[app.url_index]
                        );
                    }
                }, EventType::ForwardClicked => {
                    if app.url_index < app.visited_urls.len() - 1 {
                        app.url_index += 1;
                        app.web_view.load_uri(
                            app.visited_urls[app.url_index].as_str()
                        );
                        via_nav_btns = true;

                        info!(
                            "Forward to {}.", app.visited_urls[app.url_index]
                        );
                    }
                }, EventType::RefreshClicked => {

                }, EventType::ChangedPage => {
                    // Don't re-navigate after pressing back
                    if via_nav_btns {
                        via_nav_btns = false;
                    }

                    info!("Changed page to {}.", event.url);

                    while app.url_index < app.visited_urls.len() - 1 {
                        info!(
                            "Removing {} from forward queue.",
                            app.visited_urls[app.visited_urls.len() - 1]
                        );
                        app.visited_urls.pop();
                    }
                    app.visited_urls.push(event.url.clone());
                    app.url_index += 1;

                    app.tb_buff.set_text(event.url.as_str());
                }
            }
        }
    };
    MainContext::default().spawn_local(event_handler);

    main();
}

enum EventType {
    BackClicked,
    ForwardClicked,
    RefreshClicked,
    ChangedPage
}

struct Event {
    pub tp: EventType,
    pub url: String
}

struct AppState {
    pub web_view: WebView,
    pub tb_buff: TextBuffer,
    pub url_index: usize,
    pub visited_urls: Vec<String>
}

impl AppState {
    pub fn new(tx: Sender<Event>) -> Self {
        // Load config file
        let cfg = match load(APP_NAME) {
            Err(_) => {
                warn!("Error in config! Using defaults.");
                AppConfig::default()
            }, Ok(config) => config
        };
        let start_page = cfg.start_page.clone();

        let back_tx = tx.clone();
        let back_btn = cascade! {
            Button::with_label("←");
                ..set_border_width(cfg.margin);
                ..connect_clicked(move |_| {
                    let tx = back_tx.clone();
                    spawn(async move {
                        let _ = tx.send(Event {
                            tp: EventType::BackClicked, url: String::new()
                        }).await;
                    });
                });
        };
        let fwd_tx = tx.clone();
        let fwd_btn = cascade! {
            Button::with_label("→");
                ..set_border_width(cfg.margin);
                ..connect_clicked(move |_| {
                    let tx = fwd_tx.clone();
                    spawn(async move {
                        let _ = tx.send(Event {
                            tp: EventType::ForwardClicked, url: String::new()
                        }).await;
                    });
                });
        };
        
        let buff = TextBuffer::builder().text(&start_page).build();
        let tb = TextView::builder().hexpand(true).buffer(&buff).build();

        let refr_tx = tx.clone();
        let refr_btn = cascade! {
            Button::with_label("↺");
                ..set_border_width(cfg.margin);
                ..connect_clicked(move |_| {
                    let tx = refr_tx.clone();
                    spawn(async move {
                        let _ = tx.send(Event {
                            tp: EventType::RefreshClicked, url: String::new()
                        }).await;
                    });
                });
        };

        let web_tx = tx.clone();
        let web_view = cascade! {
            WebView::builder().build();
                ..load_uri(&start_page);
                ..connect_load_changed(move |view, load_ev| {
                    if load_ev == LoadEvent::Started {
                        let tx = web_tx.clone();
                        let txt = WebView::uri(&view).unwrap().to_string();
                        spawn(async move {
                            let _ = tx.send(Event {
                                tp: EventType::ChangedPage,
                                url: txt
                            }).await;
                        });
                    }
                });
        };
        let web_box = cascade! {
            Box::new(Orientation::Horizontal, 0);
                ..pack_start(&web_view, true, true, cfg.margin);
        };

        let view_cont = cascade! {
            Grid::builder().margin_top(cfg.margin as i32).build();
                ..attach(&back_btn, 0, 0, 1, 1);
                ..attach(&fwd_btn, 1, 0, 1, 1);
                ..attach(&tb, 2, 0, 5, 1);
                ..attach(&refr_btn, 7, 0, 1, 1);
        };
        let view = cascade! {
            Box::new(Orientation::Vertical, 0);
                ..pack_start(&view_cont, false, false, 0);
                ..pack_start(&web_box, true, true, cfg.margin);
        };

        let _window = cascade! {
            Window::new(WindowType::Toplevel);
                ..add(&view);
                ..set_title(WIN_TITLE);
                ..set_default_size(WIN_DEF_WIDTH, WIND_DEF_HEIGHT);
                ..connect_delete_event(move |_, _| {
                    main_quit();
                    Inhibit(false)
                });
                ..show_all();
        };
        //gtk::Window::set_default_icon_name("icon-name-here");

        Self {
            web_view,
            tb_buff: buff,
            url_index: 0,
            visited_urls: vec![ start_page.clone() ]
        }
    }
}
