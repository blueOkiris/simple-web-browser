/*
 * Author: Dylan Turner
 * Description: Define an interface for creating plugins
 */

use std::{
    fs::read_dir,
    sync::Arc
};
use dlopen_derive::WrapperApi;
use dlopen::wrapper::{
    Container, WrapperApi
};
use gtk::Box;

#[cfg(debug_assertions)]
const PLUGIN_DIR: &'static str = "target/debug";

// When building the final app, we'll create a "plugins" folder in install dir
#[cfg(not(debug_assertions))]
const PLUGIN_DIR: &'static str = "/opt/swb/plugins";

#[derive(WrapperApi)]
pub struct Plugin {
    name: extern fn() -> String,
    on_back_btn_clicked: extern fn(),
    on_fwd_btn_clicked: extern fn(),
    on_change_page: extern fn(url: &String),
    on_refr_btn_clicked: extern fn(),
    on_navbar_load: extern fn(navbar: &Box),
    on_window_content_load: extern fn(content: &Box)
}

pub fn load_plugins() -> Vec<Arc<Container<Plugin>>> {
    let mut plugins = Vec::new();

    let paths = read_dir(PLUGIN_DIR).unwrap();
    for path in paths {
        let fname = path.unwrap().path().display().to_string();
        if fname.ends_with(".so") {
            let plugin: Container<Plugin> = unsafe {
                Container::load(fname.clone())
            }.expect(
                (String::from("Error loading plugin from {}") + &fname.clone())
                    .as_str()
            );

            println!("Found plugin '{}' in {}", plugin.name(), fname.clone());
            plugins.push(Arc::new(plugin));
        }
    }

    plugins
}
