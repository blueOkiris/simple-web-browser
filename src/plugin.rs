/*
 * Author: Dylan Turner
 * Description: Define an interface for creating plugins
 */

use std::{
    fs::read_dir,
    sync::Arc
};
#[cfg(not(debug_assertions))]
use dirs::config_dir;
use dlopen_derive::WrapperApi;
use dlopen::wrapper::{
    Container, WrapperApi
};
use gtk::Box;

#[derive(WrapperApi)]
pub struct Plugin {
    name: extern fn() -> String
}

pub fn load_plugins() -> Vec<Arc<Container<Plugin>>> {
    let mut plugins = Vec::new();

    let paths;
    #[cfg(debug_assertions)]
    {
        paths = read_dir("target/debug").unwrap();
    }
    #[cfg(not(debug_assertions))]
    {
        let mut conf = config_dir().unwrap();
        conf.push("swb");
        conf.push("plugins");
        paths = read_dir(conf.as_os_str().to_str().unwrap()).unwrap();
    }
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
