/*
 * Author: Dylan Turner
 * Description: Define application state and create gui
 */

use std::future::Future;
use async_channel::{ unbounded, Sender };
use gtk::{
    main_quit, Inhibit, init, main,
    Button, Box, Orientation, Entry, EntryBuffer, Grid, Label, CheckButton,
    Menu, MenuItem, MenuButton,
    Window, WindowType, Align, Dialog, DialogFlags, ResponseType,
    prelude::{
        ContainerExt, ButtonExt, BoxExt, WidgetExt, GtkWindowExt, GridExt,
        MenuButtonExt, MenuShellExt, GtkMenuItemExt,
        DialogExt, EntryExt
    }, glib::{ set_program_name, set_application_name, MainContext }
};
use webkit2gtk::{ WebView, LoadEvent, traits::WebViewExt };
use serde::{ Serialize, Deserialize };
use log::{ warn, error, info, debug };
use confy::{ load, store };
use cascade::cascade;

const WIN_TITLE: &'static str = "Browse the Web";
const WIN_DEF_WIDTH: i32 = 640;
const WIN_DEF_HEIGHT: i32 = 480;
const POPUP_WIDTH: i32 = 300;
const POPUP_HEIGHT: i32 = 200;
const APP_NAME: &'static str = "swb";

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
    let app = AppState::new(tx);

    let mut via_nav_btns = false;
    let mut back_urls = vec![ app.cfg.start_page ];
    let mut fwd_urls: Vec<String> = Vec::new();

    let mut err_url = String::new();

    let event_handler = async move {
        while let Ok(event) = rx.recv().await {

            debug!("Back urls:");
            for url in back_urls.clone() {
                debug!("Back: {}.", url);
            }
            debug!("Forward urls:");
            for url in fwd_urls.clone() {
                debug!("Forward: {}.", url);
            }

            match event.tp {
                EventType::BackClicked => {
                    if back_urls.len() > 0 {
                        let new_url = back_urls.pop().unwrap();
                        fwd_urls.insert(0, new_url.clone());

                        via_nav_btns = true;
                        app.web_view.load_uri(&new_url);

                        info!("Back to {}.", new_url);

                        app.tb_buff.set_text(new_url.as_str());
                    }
                }, EventType::ForwardClicked => {
                    if fwd_urls.len() > 0 {
                        let new_url = fwd_urls.pop().unwrap();
                        back_urls.push(new_url.clone());

                        via_nav_btns = true;
                        app.web_view.load_uri(&new_url);

                        info!("Forward to {}.", new_url);

                        app.tb_buff.set_text(new_url.as_str());
                    }
                }, EventType::RefreshClicked => {
                    via_nav_btns = true;
                    app.web_view.reload();
                }, EventType::ChangedPage => {
                    info!("Changed page to {}.", event.url);

                    // Don't re-navigate after pressing back
                    if via_nav_btns {
                        via_nav_btns = false;
                        continue;
                    }

                    fwd_urls = Vec::new();
                    fwd_urls.push(event.url.clone());

                    app.tb_buff.set_text(event.url.as_str());
                }, EventType::ChangePage => {
                    app.web_view.load_uri(&event.url);
                }, EventType::FailedChangePage => {
                    warn!("Failed to change page to {}.", event.url);

                    match back_urls.clone().iter().position(
                                |e| *e == event.url
                            ) {
                        None => {},
                        Some(pos) => { back_urls.remove(pos); }
                    }
                    match fwd_urls.clone().iter().position(
                                |e| *e == event.url
                            ) {
                        None => {},
                        Some(pos) => { fwd_urls.remove(pos); }
                    }

                    if event.url == err_url {
                        warn!("Site unreachable. Loading error page.");
                        app.tb_buff.set_text("about:blank");
                        app.web_view.load_uri("about:blank");
                    } else {
                        err_url =
                            app.cfg.search_engine.replace("${}", &event.url);
                        app.web_view.load_uri(err_url.as_str());
                    }
                }, EventType::LoginRegister => {
                    /* Create a login prompt */
                    let dialog = cascade! {
                        Dialog::with_buttons(
                            Some("Sign In"), Some(&app.win),
                            DialogFlags::from_bits(1).unwrap(),
                            &[ ]
                        );
                        ..set_default_size(POPUP_WIDTH, POPUP_HEIGHT);
                        ..set_modal(true);
                        ..set_resizable(false);
                        ..add_button("Login", ResponseType::Accept);
                        ..add_button("Cancel", ResponseType::Cancel);
                    };
                    let content_area = dialog.content_area();

                    let uname_buff = EntryBuffer::new(Some(""));
                    let uname = cascade! {
                        Box::new(Orientation::Horizontal, 0);
                            ..pack_start(
                                &Label::new(Some("Username: ")),
                                false, false, app.cfg.margin
                            );..pack_start(
                                &Entry::builder()
                                    .buffer(&uname_buff).hexpand(true)
                                    .build(),
                                true, true, app.cfg.margin
                            );
                    };
                    content_area.pack_start(&uname, true, true, app.cfg.margin);

                    let pword_buff = EntryBuffer::new(Some(""));
                    let pword = cascade! {
                        Box::new(Orientation::Horizontal, 0);
                            ..pack_start(
                                &Label::new(Some("Password: ")),
                                false, false, app.cfg.margin
                            );..pack_start(
                                &Entry::builder()
                                    .buffer(&pword_buff).hexpand(true)
                                    .visibility(false)
                                    .build(),
                                true, true, app.cfg.margin
                            );
                    };
                    content_area.pack_start(&pword, true, true, app.cfg.margin);
                    
                    let remem = cascade! {
                        Box::new(Orientation::Horizontal, 0);
                            ..pack_start(
                                &CheckButton::with_label("Remember"),
                                true, true, app.cfg.margin
                            );
                    };
                    content_area.pack_start(&remem, true, true, app.cfg.margin);
                    
                    dialog.connect_response(move |view, resp| {
                        match resp {
                            ResponseType::Cancel => view.hide(),
                            ResponseType::Accept => {
                                
                            }, _ => view.hide()
                        }
                    });
                    dialog.show_all();
                }
            }
        }
    };
    MainContext::default().spawn_local(event_handler);

    main();
}

#[derive(Serialize, Deserialize)]
struct AppConfig {
    pub start_page: String,
    pub search_engine: String,
    pub local: bool,
    pub bookmarks: Vec<Vec<Vec<String>>>,
    pub username: String,
    pub pass_enc: String,
    pub margin: u32
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            start_page: String::from("https://duckduckgo.org"),
            search_engine: String::from("https://duckduckgo.com/?q=${}"),
            local: false,
            bookmarks: Vec::new(),
            username: String::new(),
            pass_enc: String::new(),
            margin: 10
        }
    }
}

enum EventType {
    BackClicked,
    ForwardClicked,
    RefreshClicked,
    ChangedPage,
    ChangePage,
    FailedChangePage,
    LoginRegister
}

struct Event {
    pub tp: EventType,
    pub url: String
}

struct AppState {
    pub win: Window,
    pub web_view: WebView,
    pub cfg: AppConfig,
    pub tb_buff: EntryBuffer
}

impl AppState {
    pub fn new(tx: Sender<Event>) -> Self {
        // Try to sync bookmarks online
        let mut temp_cfg = match load(APP_NAME) {
            Err(_) => {
                warn!("Error in config! Using defaults.");
                AppConfig::default()
            }, Ok(config) => config
        };
        if !temp_cfg.local {
            // Sync via db
            let synced_bm = Vec::new();

            if false {
                temp_cfg.bookmarks = synced_bm.clone();
                store(APP_NAME, temp_cfg).unwrap();
            }
        }

        // Load config file
        let cfg = match load(APP_NAME) {
            Err(_) => {
                warn!("Error in config! Using defaults.");
                AppConfig::default()
            }, Ok(config) => config
        };
        let start_page = cfg.start_page.clone();

        /* Create navigation bar */

        // Back button
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

        // Forward button
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

        // Search/Navigation text box
        let buff_tx = tx.clone();
        let buff = EntryBuffer::new(Some(&start_page.as_str()));
        let tb = cascade! {
            Entry::builder().hexpand(true)
                .valign(Align::Center).buffer(&buff).build();
                ..connect_activate(move |entry| {
                    let tx = buff_tx.clone();
                    let url = entry.text().to_string().clone();
                    spawn(async move {
                        let _ = tx.send(Event {
                            tp: EventType::ChangePage,
                            url
                        }).await;
                    });
                });
        };

        // Generate book marks menu
        let bookmark_menu = Menu::builder().build();
        for folder in cfg.bookmarks.clone() {
            match folder.len() {
                0 => { },
                1 => {
                    // Lots of clones bc closure expects static. Wouldn't touch
                    let bm = folder[0].clone();
                    let name = bm[0].clone();
                    let bm_url = bm[0].clone();

                    info!("Found local bookmark: {} -> '{}'.", name, bm_url);

                    let item_tx = tx.clone();
                    let item = cascade! {
                        MenuItem::with_label(name.as_str());
                            ..connect_activate(move |_| {
                                let tx = item_tx.clone();
                                let url = bm_url.clone();
                                spawn(async move {
                                    let _ = tx.send(Event {
                                        tp: EventType::ChangePage,
                                        url
                                    }).await;
                                });
                            });
                    };
                    bookmark_menu.append(&item);
                }, _ => {
                    let fldr_name = folder[0][0].clone();
                    let sub_menu = Menu::builder().build();

                    for i in 1..folder.len() {
                        let fldr_clone = folder.clone();
                        let bookmark = fldr_clone[i].clone();

                        let name = bookmark[0].clone();
                        let bm_url = bookmark[1].clone();

                        info!(
                            "Found local bookmark: {}/{} -> '{}'.",
                            fldr_name, name, bm_url
                        );

                        let item_tx = tx.clone();
                        let item = cascade! {
                            MenuItem::with_label(name.as_str());
                                ..connect_activate(move |_| {
                                    let tx = item_tx.clone();
                                    let url = bm_url.clone();
                                    spawn(async move {
                                        let _ = tx.send(Event {
                                            tp: EventType::ChangePage,
                                            url
                                        }).await;
                                    });
                                });
                        };
                        sub_menu.append(&item);
                    }

                    sub_menu.show_all();
                    let item = cascade! {
                        MenuItem::with_label(fldr_name.as_str());
                            ..set_submenu(Some(&sub_menu));
                    };
                    bookmark_menu.append(&item);
                }
            }
        }
        bookmark_menu.show_all();
        let bm_btn = cascade! {
            MenuButton::builder().label("@").build();
                ..set_border_width(cfg.margin);
                ..set_popup(Some(&bookmark_menu));
        };

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

        /* Create page view */
        let web_tx1 = tx.clone();
        let web_tx2 = tx.clone();
        let web_view = cascade! {
            WebView::builder().build();
                ..load_uri(&start_page);
                ..connect_load_changed(move |view, load_ev| {
                    if load_ev == LoadEvent::Started {
                        let tx = web_tx1.clone();
                        let txt = WebView::uri(&view).unwrap().to_string();
                        spawn(async move {
                            let _ = tx.send(Event {
                                tp: EventType::ChangedPage,
                                url: txt
                            }).await;
                        });
                    }
                });
                ..connect_load_failed(move |_, load_ev, uri, _| {
                    if load_ev == LoadEvent::Started {
                        let tx = web_tx2.clone();
                        let url = String::from(uri);
                        spawn(async move {
                            let _ = tx.send(Event {
                                tp: EventType::FailedChangePage,
                                url
                            }).await;
                        });
                        return true;
                    }
                    false
                });
        };
        let web_box = cascade! {
            Box::new(Orientation::Horizontal, 0);
                ..pack_start(&web_view, true, true, cfg.margin);
        };

        /* Put it all together */
        let view_cont = cascade! {
            Grid::builder().build();
                ..attach(&back_btn, 0, 0, 1, 1);
                ..attach(&fwd_btn, 1, 0, 1, 1);
                ..attach(&tb, 2, 0, 5, 1);
                ..attach(&bm_btn, 7, 0, 1, 1);
                ..attach(&refr_btn, 8, 0, 1, 1);
        };

        // Sync popup button
        if cfg.local {
            let sync_tx = tx.clone();
            let sync_btn = cascade! {
                Button::with_label("↨");
                    ..set_border_width(cfg.margin);
                    ..connect_clicked(move |_| {
                        let tx = sync_tx.clone();
                        spawn(async move {
                            let _ = tx.send(Event {
                                tp: EventType::LoginRegister,
                                url: String::new()
                            }).await;
                        });
                    });
            };
            view_cont.attach(&sync_btn, 9, 0, 1, 1);
        }

        let view = cascade! {
            Box::new(Orientation::Vertical, 0);
                ..pack_start(&view_cont, false, false, 0);
                ..pack_end(&web_box, true, true, cfg.margin);
        };
        let win = cascade! {
            Window::new(WindowType::Toplevel);
                ..add(&view);
                ..set_title(WIN_TITLE);
                ..set_default_size(WIN_DEF_WIDTH, WIN_DEF_HEIGHT);
                ..connect_delete_event(move |_, _| {
                    main_quit();
                    Inhibit(false)
                });
                ..show_all();
        };
        //gtk::Window::set_default_icon_name("icon-name-here");

        Self {
            win,
            web_view,
            cfg,
            tb_buff: buff
        }
    }
}
