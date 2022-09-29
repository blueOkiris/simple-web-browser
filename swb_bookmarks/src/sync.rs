/*
 * Author: Dylan Turner
 * Description: State and functions for logging in or already being logged in
 */

use std::error::Error;
use gtk::{
    Menu, MenuItem,
    prelude::{
        MenuShellExt, GtkMenuItemExt
    }
};
use serde_json::{
    to_string, from_str
};
use serde::{
    Serialize, Deserialize
};
use reqwest::get;
use tokio::runtime::Runtime;
use crate::{
    config::Config,
    MSG_QUEUE
};

// Copied from db/src/db.rs
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BookmarkCollection {
    pub name: String,
    pub bms: Vec<(String, String)>,
    pub subfldrs: Vec<BookmarkCollection>
}

impl BookmarkCollection {
    pub fn get_subfldr_menu_item(&self) -> Vec<MenuItem> {
        let mut menu_items = Vec::new();
        for subfldr in self.subfldrs.iter() {
            let fldr = MenuItem::builder()
                .label(&subfldr.name.clone())
                .hexpand(true).vexpand(false)
                .build();
            let fldr_menu = Menu::builder().build();
            
            let subsubfldr_items = subfldr.get_subfldr_menu_item();
            for menu_item in subsubfldr_items {
                fldr_menu.append(&menu_item);
            }

            for bm in subfldr.bms.iter() {
                let bm_item = MenuItem::builder()
                    .label(&bm.0.clone())
                    .hexpand(true).vexpand(false)
                    .build();
                let name_clone = bm.0.clone();
                bm_item.connect_activate(move |_menu_item| {
                    unsafe {
                        if MSG_QUEUE.clone().is_none() {
                            MSG_QUEUE = Some(Vec::new())
                        }
                        let mut queue = MSG_QUEUE.clone().unwrap();
                        queue.push((
                            String::from("WEBKIT_REDIRECT"),
                            name_clone.clone()
                        ));
                        MSG_QUEUE = Some(queue);
                    }
                });
                fldr_menu.append(&bm_item);
            }
            fldr.set_submenu(Some(&fldr_menu));

            menu_items.push(fldr);
        }

        menu_items
    }
}

pub fn login(email: &str, pword: &str) -> Result<(), Box<dyn Error>> {
    let runtime = Runtime::new().unwrap();
    let fut = login_async(email, pword);
    runtime.block_on(fut)
}

async fn login_async(email: &str, pword: &str) -> Result<(), Box<dyn Error>> {
    let login_attempt_url = String::from("http://blueokiris.com:9420/login/") + email + "/" + pword;
    let attempt_res_text = get(login_attempt_url).await?.text().await?;

    if attempt_res_text == "success" {
        Ok(())
    } else {
        Err(format!("Error logging in: {}", attempt_res_text).as_str().into())
    }
}

pub fn register(email: &str, pword: &str) -> Result<(), Box<dyn Error>> {
    let runtime = Runtime::new().unwrap();
    let fut = register_async(email, pword);
    runtime.block_on(fut)
}

async fn register_async(email: &str, pword: &str) -> Result<(), Box<dyn Error>> {
    let reg_attempt_url =
        String::from("http://blueokiris.com:9420/register/") + email + "/" + pword;
    let attempt_res_text = get(reg_attempt_url).await?.text().await?;
    if attempt_res_text == "success" {
        Ok(())
    } else {
        Err(format!("Error registering: {}", attempt_res_text).as_str().into())
    }
}

pub fn get_bookmarks() -> Result<BookmarkCollection, Box<dyn Error>> {
    let cfg = Config::get_global();
    if !cfg.logged_in {
        Err("Can't sync bookmarks! Not logged in.")?
    }

    let runtime = Runtime::new().unwrap();
    let fut = get_bookmarks_async(&cfg.email, &cfg.pword);
    runtime.block_on(fut)
}

async fn get_bookmarks_async(
        email: &str, pword: &str) -> Result<BookmarkCollection, Box<dyn Error>> {
    let reg_attempt_url =
        String::from("http://blueokiris.com:9420/bookmarks/") + email + "/" + pword;
    let attempt_res_text = get(reg_attempt_url).await?.text().await?;
    if attempt_res_text.starts_with("success") {
        let bm_str = attempt_res_text.split_at(7).1;
        let bm_col = from_str(bm_str)?;
        Ok(bm_col)
    } else {
        Err(format!("Error retrieving bookmarks: {}", attempt_res_text).as_str().into())
    }
}

