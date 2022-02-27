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

static mut CONFIG: Option<Config> = None;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub stay_logged_in: bool,
    pub email: String,
    pub pword: String,
    pub logged_in: bool
}

impl Default for Config {
    fn default() -> Config {
        Config {
            stay_logged_in: false,
            email: String::new(),
            pword: String::new(),
            logged_in: false
        }
    }
}

impl Config {
    pub fn get_global() -> Config {
        unsafe {
            if CONFIG.is_none() {
                match load("SWB_BOOKMARKS") {
                    Err(err) => {
                        println!("Error '{}' in config! Using defaults.", err);
                        CONFIG = Some(Config::default());
                        store("SWB_BOOKMARKS", CONFIG.clone().unwrap())
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
            store("SWB_BOOKMARKS", cfg).unwrap();
        }
    }
}
