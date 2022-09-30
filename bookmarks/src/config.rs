/*
 * Author: Dylan Turner
 * Description: Login stuff requires storing data. Manage it here
 */

use serde::{
    Serialize, Deserialize
};
use confy::{
    load, store
};
use crate::sync::BookmarkCollection;

static mut CONFIG: Option<Config> = None;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub stay_logged_in: bool,
    pub email: String,
    pub pword: String,
    pub logged_in: bool,
    pub bm_collection: BookmarkCollection
}

impl Default for Config {
    fn default() -> Config {
        Config {
            stay_logged_in: false,
            email: String::new(),
            pword: String::new(),
            logged_in: false,
            bm_collection: BookmarkCollection {
                name: String::new(),
                bms: Vec::new(),
                subfldrs: Vec::new()
            }
        }
    }
}

impl Config {
    pub fn get_global() -> Config {
        unsafe {
            if CONFIG.is_none() {
                match load("swb_bookmarks") {
                    Err(err) => {
                        println!("Error '{}' in config! Using defaults.", err);
                        CONFIG = Some(Config::default());
                        store("swb_bookmarks", CONFIG.clone().unwrap())
                            .unwrap();
                    }, Ok(config) => CONFIG = Some(config)
                }
            }

            CONFIG.clone().unwrap()
        }
    }

    pub fn set_global(config: Config) {
        unsafe {
            CONFIG = Some(config);
        }
    }

    pub fn store_global() {
        unsafe {
            let mut cfg = CONFIG.clone().unwrap();
            cfg.logged_in = false; // Never log in by default
            store("swb_bookmarks", cfg).unwrap();
        }
    }
}
