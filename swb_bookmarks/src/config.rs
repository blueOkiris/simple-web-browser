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

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub stay_logged_in: bool
}

impl Default for Config {
    fn default() -> Config {
        Config {
            stay_logged_in: false
        }
    }
}

impl Config {
    pub fn get_global() -> Config {
        unsafe {
            if CONFIG.is_none() {
                match load("SWB_BOOKMARKS") {
                    Err(_err) => {
                        println!("Error in config! Using defaults.");
                        CONFIG = Some(Config::default())
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
            store("SWB_BOOKMARKS", CONFIG.clone().unwrap()).unwrap();
        }
    }
}
