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
use serde_json::from_str;
use serde::{
    Serialize, Deserialize
};
use reqwest::get;
use tokio::runtime::Runtime;
use crate::{
    config::Config,
    MSG_QUEUE
};

const BM_SERVER: &'static str = "http://dylan-turner.mynetgear.com:9420/";

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
                let url_clone = bm.1.clone();
                bm_item.connect_activate(move |_menu_item| {
                    unsafe {
                        if MSG_QUEUE.clone().is_none() {
                            MSG_QUEUE = Some(Vec::new())
                        }
                        let mut queue = MSG_QUEUE.clone().unwrap();
                        queue.push((
                            String::from("WEBKIT_REDIRECT"),
                            url_clone.clone()
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

    pub fn remove(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        if path == "" {
            Err("Error: Cannot remove empty path!")?
        }

        let pieces = path.split('/').collect::<Vec<&str>>();
        if pieces.len() < 2 {
            for i in 0..self.bms.len() {
                if self.bms[i].0 == path {
                    self.bms.remove(i);
                    return Ok(());
                }
            }

            for i in 0..self.subfldrs.len() {
                if self.subfldrs[i].name == path {
                    self.subfldrs.remove(i);
                    return Ok(());
                }
            }
        } else {
            for i in 0..self.subfldrs.len() {
                if self.subfldrs[i].name == pieces[0] {
                    let mut subpath = String::new();
                    for i in 1..pieces.len() {
                        subpath.push_str(pieces[i]);
                        subpath.push('/');
                    }
                    subpath.pop();

                    return self.subfldrs[i].remove(subpath.as_str());
                }
            }
        }

        Err("Error: Invalid path provided!")?
    }

    /*
     * Create a JSON representation of what the database stores bookmarks and stuff as
     *
     * Reference:
     *   Collection: { name: String, bms <Vec(String, String)>, subfolders: Vec<Collection> }
     */
    pub fn to_string(&self) -> String {
        let mut subfldrs = String::from("[");
        for subfldr in self.subfldrs.iter() {
            subfldrs.push_str(subfldr.to_string().as_str());
            subfldrs.push(',');
        }
        if subfldrs.len() > 1 {
            subfldrs.pop();
        }
        subfldrs.push(']');

        let mut bms = String::from("[");
        for bm in self.bms.iter() {
            bms.push_str("[\"");
            bms.push_str(bm.0.as_str());
            bms.push_str("\",\"");
            bms.push_str(bm.1.as_str());
            bms.push_str("\"],");
        }
        if bms.len() > 1 {
            bms.pop();
        }
        bms.push(']');
        
        format!(
            "{{\"name\":\"{}\",\"bms\":{},\"subfldrs\":{}}}",
            self.name, bms, subfldrs
        ).replace("https://", "")
    }
}

pub fn login(email: &str, pword: &str) -> Result<(), Box<dyn Error>> {
    let runtime = Runtime::new().unwrap();
    let fut = login_async(email, pword);
    runtime.block_on(fut)
}

async fn login_async(email: &str, pword: &str) -> Result<(), Box<dyn Error>> {
    let login_attempt_url = format!("{}/login/{}/{}", BM_SERVER, email, pword);
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
    let reg_attempt_url = format!("{}/register/{}/{}", BM_SERVER, email, pword);
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
    let bm_lookup_attempt_url = format!("{}/bookmarks/{}/{}", BM_SERVER, email, pword);
    let attempt_res_text = get(bm_lookup_attempt_url).await?.text().await?;
    if attempt_res_text.starts_with("success") {
        let bm_str = attempt_res_text.split_at(7).1;
        let bm_col = from_str(bm_str)?;
        Ok(bm_col)
    } else {
        Err(format!("Error retrieving bookmarks: {}", attempt_res_text).as_str().into())
    }
}

pub fn set_bookmarks(coll: &BookmarkCollection) -> Result<(), Box<dyn Error>> {
    let cfg = Config::get_global();
    if !cfg.logged_in {
        Err("Can't save bookmarks changes! Not logged in.")?
    }

    let runtime = Runtime::new().unwrap();
    let fut = set_bookmarks_async(&cfg.email, &cfg.pword, &coll);
    runtime.block_on(fut)
}

async fn set_bookmarks_async(
        email: &str, pword: &str, coll: &BookmarkCollection) -> Result<(), Box<dyn Error>> {
    let set_attempt_url = format!(
        "{}/set_bookmarks/{}/{}/{}", BM_SERVER, email, pword, coll.to_string()
    );
    let attempt_res_text = get(set_attempt_url).await?.text().await?;
    if attempt_res_text == "success" {
        Ok(())
    } else {
        Err(format!("Failed to set bookmarks: {}", attempt_res_text).as_str().into())
    }
}

pub fn request_pword_reset(email: &str) -> Result<(), Box<dyn Error>> {
    let runtime = Runtime::new().unwrap();
    let fut = request_pword_reset_async(email);
    runtime.block_on(fut)
}

async fn request_pword_reset_async(email: &str) -> Result<(), Box<dyn Error>> {
    let reset_req_attempt_url = format!("{}/req_pass_rst/{}", BM_SERVER, email);
    let attempt_res_text = get(reset_req_attempt_url).await?.text().await?;

    if attempt_res_text == "success" {
        Ok(())
    } else {
        Err(format!("Error requesting reset request: {}", attempt_res_text).as_str().into())
    }
}

pub fn reset_password(email: &str, new_password: &str, code: &str) -> Result<(), Box<dyn Error>> {
    let runtime = Runtime::new().unwrap();
    let fut = reset_password_async(email, new_password, code);
    runtime.block_on(fut)
}

async fn reset_password_async(
        email: &str, new_password: &str, code: &str) -> Result<(), Box<dyn Error>> {
    let reset_password_attempt_url = format!(
        "{}/change_pass/{}/{}/{}", BM_SERVER, email, code, new_password
    );
    let attempt_res_text = get(reset_password_attempt_url).await?.text().await?;

    if attempt_res_text == "success" {
        Ok(())
    } else {
        Err(format!("Error reset request: {}", attempt_res_text).as_str().into())
    }
}

