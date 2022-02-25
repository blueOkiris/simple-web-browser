/*
 * Author: Dylan Turner
 * Description: Define application state and create gui
 */

use std::future::Future;
use async_channel::{ unbounded, Sender };
use gtk::{
    main_quit, Inhibit, init, main,
    Button, Box, Orientation, Entry, EntryBuffer, Grid, Dialog,
    Menu, MenuItem, MenuButton,
    Window, WindowType, Align, ResponseType,
    prelude::{
        ContainerExt, ButtonExt, BoxExt, WidgetExt, GtkWindowExt, GridExt,
        MenuButtonExt, MenuShellExt, GtkMenuItemExt,
        DialogExt, EntryExt
    }, glib::{ set_program_name, set_application_name, MainContext }
};
use webkit2gtk::{
    WebView, LoadEvent, WebContext, traits::{ WebViewExt, WebContextExt }
};
use serde::{ Serialize, Deserialize };
use log::{ warn, error, info, debug };
use confy::{ load, store };
use cascade::cascade;

use crate::gui::{
    create_error_popup, create_login_dialog, create_success_popup,
    create_bookmark_add_dialog, create_bookmark_delete_dialog,
    create_folder_add_dialog
};
use crate::db::{ login, register };

const WIN_TITLE: &'static str = "Browse the Web";
const WIN_DEF_WIDTH: i32 = 640;
const WIN_DEF_HEIGHT: i32 = 480;
const APP_NAME: &'static str = "swb";
const EXTENSION_DIR: &'static str = "/home/dylan/.swb-extensions";

// Spawns a task on default executor, without waiting to complete
fn spawn<F>(future: F) where F: Future<Output = ()> + 'static {
    MainContext::default().spawn_local(future);
}

#[derive(PartialEq)]
enum EventType {
    BackClicked,
    ForwardClicked,
    RefreshClicked,
    ChangedPage, // On a page going through
    ChangePage, // Sent by a but TO change page
    FailedChangePage,
    LoginRegister,
    AddFolder,
    AddBookmark, // Prompt for adding a bookmark
    DeleteBookmarkOrFolder,
    OpenBlockMenu
}

struct Event {
    pub tp: EventType,
    pub url: String
}

enum ModAppType {
    CreateBookmark,
    ChangeTbBuffer
}

struct ModApp {
    tp: ModAppType,
    tb_new_val: String,
    name_buff: Option<EntryBuffer>,
    fldr_buff: Option<EntryBuffer>,
    dialog: Option<Dialog>,
    url: String
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

    let event_handler = async move {
        let mut app = AppState::new(tx);
        
        // Store some state so the application can go back and forth
        let mut via_nav_btns = false;
        let mut back_urls = vec![ app.cfg.start_page.clone() ];
        let mut fwd_urls: Vec<String> = Vec::new();

        // We try to search first, but we store it so if we fail again fail
        let mut err_url = String::new();

        let quit = false;
        while !quit {
            let event = rx.recv().await.unwrap();
            let new_ev = match event.tp {
                EventType::BackClicked => {
                    if back_urls.len() > 1 {
                        let new_url = back_urls.pop().unwrap();
                        fwd_urls.push(new_url.clone());

                        via_nav_btns = true;
                        app.web_view.load_uri(
                            // account for current being stored
                            &back_urls[back_urls.len() - 1]
                        );

                        info!("Back to {}.", back_urls[back_urls.len() - 1]);
                    }
                    Some(ModApp {
                        tp: ModAppType::ChangeTbBuffer,
                        tb_new_val: back_urls[back_urls.len() - 1].clone(),
                        name_buff: None, fldr_buff: None, dialog: None,
                        url: String::new()
                    })
                }, EventType::ForwardClicked => {
                    if fwd_urls.len() > 0 {
                        let new_url = fwd_urls.pop().unwrap();
                        back_urls.push(new_url.clone());

                        via_nav_btns = true;
                        app.web_view.load_uri(&new_url);

                        info!("Forward to {}.", new_url);

                        Some(ModApp {
                            tp: ModAppType::ChangeTbBuffer,
                            tb_new_val: new_url,
                            name_buff: None, fldr_buff: None, dialog: None,
                            url: String::new()
                        })
                    } else {
                        None
                    }
                }, EventType::RefreshClicked => {
                    via_nav_btns = true;
                    app.web_view.reload();
                    None
                }, EventType::OpenBlockMenu => {
                    err_url = app.tb_buff.text().clone();
                    app.web_view.load_uri("blockit://settings");
                    via_nav_btns = true;
                    None
                }, EventType::ChangedPage => {
                    info!("Changed page to {}.", event.url);

                    // Don't re-navigate after pressing back
                    if via_nav_btns {
                        via_nav_btns = false;
                        continue;
                    }

                    fwd_urls = Vec::new();
                    back_urls.push(event.url.clone());

                    app.tb_buff.set_text(event.url.as_str());
                    None
                }, EventType::ChangePage => {
                    app.web_view.load_uri(&event.url);
                    None
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
                        app.web_view.load_uri("about:blank");

                        Some(ModApp {
                            tp: ModAppType::ChangeTbBuffer,
                            tb_new_val: String::from("about:blank"),
                            name_buff: None, fldr_buff: None, dialog: None,
                            url: String::new()
                        })
                    } else {
                        err_url =
                            app.cfg.search_engine.replace("${}", &event.url);
                        app.web_view.load_uri(err_url.as_str());
                        None
                    }
                }, EventType::LoginRegister => {
                    /* Create a login prompt */
                    let login_w = create_login_dialog(
                        app.cfg.margin, &(app.win.clone()));
                    login_w.dialog.connect_response(move |view, resp| {
                        let email = login_w.email_buff.text().clone();
                        let password = login_w.pword_buff.text().clone();
                        match resp {
                            ResponseType::Cancel => view.hide(),
                            ResponseType::Accept => { // Login
                                info!("Attempting to login to sync.");

                                let try_login = login(&email, &password);

                                match try_login {
                                    Err(err) => create_error_popup(
                                        &format!("{}", err)
                                    ), Ok(_) => {
                                        view.hide();
                                        create_success_popup(&String::from(
                                            "Succesfully logged in."
                                        ));

                                        // Set up bookmark sync and stuff
                                    }
                                }
                            }, ResponseType::Apply => { // Register
                                info!("Attempting to register new user.");

                                let try_register = register(&email, &password);

                                match try_register {
                                    Err(err) => create_error_popup(
                                        &format!("{}", err)
                                    ), Ok(_) => {
                                        view.hide();
                                        create_success_popup(&String::from(
                                            "Succesfully registered."
                                        ));

                                        // Set up bookmark sync and stuff
                                    }
                                }
                            }, _ => view.hide()
                        }
                    });
                    login_w.dialog.show_all();
                    None
                }, EventType::AddBookmark => {
                    let bm_win = create_bookmark_add_dialog(
                        app.cfg.margin, &(app.win.clone()),
                        &(app.tb_buff.text().clone())
                    );
                    bm_win.dialog.show_all();
                    Some(ModApp {
                        tp: ModAppType::CreateBookmark,
                        tb_new_val: String::new(),
                        name_buff: Some(bm_win.name),
                        fldr_buff: Some(bm_win.fldr),
                        url: bm_win.url,
                        dialog: Some(bm_win.dialog)
                    })
                }, EventType::AddFolder => {
                    let bm_win = create_folder_add_dialog(
                        app.cfg.margin, &(app.win.clone()),
                        &(app.tb_buff.text().clone())
                    );
                    bm_win.dialog.connect_response(move |view, resp| {
                        match resp {
                            ResponseType::Cancel => view.hide(),
                            _ => view.hide()
                        }
                    });
                    bm_win.dialog.show_all();
                    None
                }, EventType::DeleteBookmarkOrFolder => {
                    let bm_win = create_bookmark_delete_dialog(
                        app.cfg.margin, &(app.win.clone())
                    );
                    bm_win.dialog.connect_response(move |view, resp| {
                        match resp {
                            ResponseType::Cancel => view.hide(),
                            _ => view.hide()
                        }
                    });
                    bm_win.dialog.show_all();
                    None
                }
            };
            
            // App state borrowing is finicky, so wait to modify till end
            match new_ev {
                None => {},
                Some(mod_app) => {
                    match mod_app.tp {
                        ModAppType::CreateBookmark => {
                            let dialog = mod_app.dialog.unwrap();
                            let (tx2, rx2) = unbounded();
                            dialog.connect_response(move |view, resp| {
                                match resp {
                                    ResponseType::Accept => {
                                        let tx2_clone = tx2.clone();
                                        spawn(async move {
                                            let _ = tx2_clone.send(true).await;
                                        });
                                        view.hide();
                                    }, _ => view.hide()
                                }
                            });
                            let _t = rx2.recv().await;
                            match app.add_bookmark(
                                        &mod_app.name_buff.unwrap().text(),
                                        &mod_app.fldr_buff.unwrap().text(),
                                        &mod_app.url
                                    ) {
                                Ok(_) => {},
                                Err(msg) => create_error_popup(&msg)
                            }
                        }, ModAppType::ChangeTbBuffer => {
                            app.tb_buff.set_text(mod_app.tb_new_val.as_str());
                        }
                    }
                }
            }

            debug!("Back urls:");
            for url in back_urls.clone() {
                debug!("Back: {}.", url);
            }
            debug!("Forward urls:");
            for url in fwd_urls.clone() {
                debug!("Forward: {}.", url);
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
            start_page: String::from("https://duckduckgo.com"),
            search_engine: String::from("https://duckduckgo.com/?q=${}"),
            local: true,
            bookmarks: Vec::new(),
            username: String::new(),
            pass_enc: String::new(),
            margin: 10
        }
    }
}

struct AppState {
    pub win: Window,
    pub web_view: WebView,
    pub cfg: AppConfig,
    pub tb_buff: EntryBuffer,
    view: Box,
    tx: Sender<Event>
}

impl AppState {
    pub fn new(tx: Sender<Event>) -> Self {
        AppState::try_sync_bookmarks();

        let cfg = AppState::load_config();
        let start_page = cfg.start_page.clone();

        /* Create navigation bar */
        let (view, web_view, tb_buff) = AppState::rebuild_gui(
            tx.clone(), &start_page, &cfg
        );

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

        Self { win, web_view, cfg, tb_buff, view, tx }
    }

    pub fn add_bookmark(
            &mut self, name: &String, fldr: &String,
            url: &String) -> Result<(), String> {
        let mut fldr_ind: i64 = -1;
        for i in 0..self.cfg.bookmarks.len() {
            let folder = self.cfg.bookmarks[i].clone();
            match folder.len() {
                0 => {},
                1 => {
                    let bm = folder[0].clone();
                    if bm[0] == name.clone() {
                        return Err(format!(
                            "Bookmark '{}' arleady exists!",
                            name
                        ));
                    }
                }, _ => {
                    if folder[0][0].clone() == fldr.clone() {
                        fldr_ind = i as i64; // Found folder to place it in
                    }
                    for j in 2..folder.len() {
                        let bm = folder[j].clone();
                        if bm[0] == name.clone() {
                            return Err(format!(
                                "Bookmark '{}' arleady exists!",
                                name
                            ));
                        }
                    }
                }
            }
        }

        if fldr.clone() != String::new() && fldr_ind != -1 {
            return Err(format!("Folder '{}' does not exist!", fldr));
        }
        
        if fldr_ind != -1 {
            self.cfg.bookmarks[fldr_ind as usize].push(
                vec![ name.clone(), url.clone() ]
            );
        } else {
            self.cfg.bookmarks.push(vec! [
                vec![ name.clone(), url.clone() ]
            ]);
        }

        store(APP_NAME, &self.cfg).unwrap();
        
        self.win.remove(&self.view);
        let (view, _web_view, _tb_buff) = AppState::rebuild_gui(
            self.tx.clone(), &self.cfg.start_page, &self.cfg
        );
        self.win.add(&view);
        self.win.show_all();

        Ok(())
    }

    fn rebuild_gui(
            tx: Sender<Event>, start_page: &String,
            cfg: &AppConfig) -> (Box, WebView, EntryBuffer) {
        // Back button
        let back_btn = Button::with_label("←");
        back_btn.set_border_width(cfg.margin);
        let back_tx = tx.clone();
        back_btn.connect_clicked(move |_| {
            let tx = back_tx.clone();
            spawn(async move {
                let _ = tx.send(Event {
                    tp: EventType::BackClicked, url: String::new()
                }).await;
            });
        });

        // Forward button
        let fwd_btn = Button::with_label("→");
        fwd_btn.set_border_width(cfg.margin);
        let fwd_tx = tx.clone();
        fwd_btn.connect_clicked(move |_| {
            let tx = fwd_tx.clone();
            spawn(async move {
                let _ = tx.send(Event {
                    tp: EventType::ForwardClicked, url: String::new()
                }).await;
            });
        });

        // Search/Navigation text box
        let buff = EntryBuffer::new(Some(&start_page.as_str()));
        let tb = Entry::builder()
            .hexpand(true).valign(Align::Center).buffer(&buff).build();
        let tb_tx = tx.clone();
        tb.connect_activate(move |entry| {
            let tx = tb_tx.clone();
            let url = entry.text().to_string().clone();
            spawn(async move {
                let _ = tx.send(Event {
                    tp: EventType::ChangePage,
                    url
                }).await;
            });
        });

        let bm_btn = AppState::create_bookmark_btn(&cfg, tx.clone());

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

        let block_tx = tx.clone();
        let block_btn = cascade! {
            Button::with_label("⯃");
                ..set_border_width(cfg.margin);
                ..connect_clicked(move |_| {
                    let tx = block_tx.clone();
                    spawn(async move {
                        let _ = tx.send(Event {
                            tp: EventType::OpenBlockMenu, url: String::new()
                        }).await;
                    });
                });
        };

        /* Create page view */
        let web_tx1 = tx.clone();
        let web_tx2 = tx.clone();
        let ctx = cascade! {
            WebContext::default().unwrap();
                ..set_web_extensions_directory(EXTENSION_DIR);
        };
        let web_view = cascade! {
            WebView::builder().web_context(&ctx).build();
                ..load_uri(&start_page);
                ..connect_load_changed(move |view, load_ev| {
                    if load_ev == LoadEvent::Finished {
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
                ..attach(&block_btn, 9, 0, 1, 1);
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

        // Main view
        let view = cascade! {
            Box::new(Orientation::Vertical, 0);
                ..pack_start(&view_cont, false, false, 0);
                ..pack_end(&web_box, true, true, cfg.margin);
        };

        (view, web_view, buff)
    }

    // Generate book marks menu
    fn create_bookmark_btn(cfg: &AppConfig, tx: Sender<Event>) -> MenuButton {
        let bookmark_menu = Menu::builder().build();

        bookmark_menu.append(&AppState::create_add_bookmark_btn(tx.clone()));
        bookmark_menu.append(&AppState::create_add_folder_btn(tx.clone()));
        bookmark_menu.append(&AppState::create_delete_bookmark_btn(tx.clone()));

        for folder in cfg.bookmarks.clone() {
            match folder.len() {
                0 => { },
                1 => {
                    // Lots of clones bc closure expects static. Wouldn't touch
                    let bm = folder[0].clone();
                    let name = bm[0].clone();
                    let bm_url = bm[0].clone();

                    info!("Found local bookmark: {} -> '{}'.", name, bm_url);

                    let item = AppState::create_bookmark_item(
                        tx.clone(), &name, &bm_url
                    );
                    bookmark_menu.append(&item);
                }, _ => {
                    let fldr_name = folder[0][0].clone();
                    let sub_menu = Menu::builder().build();

                    for i in 2..folder.len() {
                        let fldr_clone = folder.clone();
                        let bookmark = fldr_clone[i].clone();

                        let name = bookmark[0].clone();
                        let bm_url = bookmark[1].clone();

                        info!(
                            "Found local bookmark: {}/{} -> '{}'.",
                            fldr_name, name, bm_url
                        );

                        let item = AppState::create_bookmark_item(
                            tx.clone(), &name, &bm_url
                        );
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

        bm_btn
    }

    fn load_config() -> AppConfig {
        match load(APP_NAME) {
            Err(_) => {
                warn!("Error in config! Using defaults.");
                AppConfig::default()
            }, Ok(config) => config
        }
    }

    fn try_sync_bookmarks() {
        let mut temp_cfg = AppState::load_config(); // Load local
        if !temp_cfg.local {
            // TODO: Sync via db
            let synced_bm = Vec::new();

            if false { // TODO: Check for changes
                temp_cfg.bookmarks = synced_bm.clone();
                store(APP_NAME, temp_cfg).unwrap();
            }
        }
    }

    fn create_bookmark_item(
            tx: Sender<Event>, name: &String, bm_url: &String) -> MenuItem {
        let item_tx = tx.clone();
        let bm_url_cpy = bm_url.clone();
        let item = cascade! {
            MenuItem::with_label(name.as_str());
                ..connect_activate(move |_| {
                    let tx = item_tx.clone();
                    let url = bm_url_cpy.clone();
                    spawn(async move {
                        let _ = tx.send(Event {
                            tp: EventType::ChangePage,
                            url
                        }).await;
                    });
                });
        };
        item
    }

    fn create_add_bookmark_btn(tx: Sender<Event>) -> MenuItem {
        let item_tx = tx.clone();
        let item = cascade! {
            MenuItem::with_label("<Add Bookmark>");
                ..connect_activate(move |_| {
                    let tx = item_tx.clone();
                    spawn(async move {
                        let _ = tx.send(Event {
                            tp: EventType::AddBookmark,
                            url: String::new()
                        }).await;
                    });
                });
        };
        item
    }

    fn create_add_folder_btn(tx: Sender<Event>) -> MenuItem {
        let item_tx = tx.clone();
        let item = cascade! {
            MenuItem::with_label("<Add Bookmarks Folder>");
                ..connect_activate(move |_| {
                    let tx = item_tx.clone();
                    spawn(async move {
                        let _ = tx.send(Event {
                            tp: EventType::AddFolder,
                            url: String::new()
                        }).await;
                    });
                });
        };
        item
    }

    fn create_delete_bookmark_btn(tx: Sender<Event>) -> MenuItem {
        let item_tx = tx.clone();
        let item = cascade! {
            MenuItem::with_label("<Delete Bookmark or Folder>");
                ..connect_activate(move |_| {
                    let tx = item_tx.clone();
                    spawn(async move {
                        let _ = tx.send(Event {
                            tp: EventType::DeleteBookmarkOrFolder,
                            url: String::new()
                        }).await;
                    });
                });
        };
        item
    }
}
