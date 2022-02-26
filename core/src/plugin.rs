/*
 * Author: Dylan Turner
 * Description: Define an interface for creating plugins
 */

use std::fs::read_dir;
use libloading::{ Library, Symbol, Error };

#[cfg(debug_assertions)]
const PLUGIN_DIR: &'static str = "target/debug";

// When building the final app, we'll create a "plugins" folder in install dir
#[cfg(not(debug_assertions))]
const PLUGIN_DIR: &'static str = "plugins";

/*
 * This is the main thing right here!
 * If you want to make a plugin, it has to implement all of these methods
 * even if it chooses to do nothing in them
 * 
 * If it's missing a plugin, default behavior (usually nothing) is used
 * This means updates adding functions won't break your plugin
 * That is EXCEPT for "name" which MUST exist. It's a 
 */

type NameProducer = unsafe fn(&mut String);

pub struct Plugin {
    lib: Library,
    name: String
}

impl Plugin {
    pub fn new(fname: String) -> Self {
        let lib = unsafe { Library::new(fname.clone()).unwrap() };

        let mut name: String = String::new();
        let get_name_func: Result<Symbol<NameProducer>, Error> = unsafe {
            lib.get(b"name")
        };
        match get_name_func {
            Err(err) => {
                name = fname.clone();
                println!("Error calling name() in {}: {}", fname.clone(), err);
            }, Ok(get_name) => unsafe { get_name(&mut name) }
        }

        Self { lib, name }
    }

    pub fn from_folder() -> Vec<Plugin> {
        let mut libs = Vec::new();

        let paths = read_dir(PLUGIN_DIR).unwrap();
        for path in paths {
            let fname = path.unwrap().path().display().to_string();
            if fname.ends_with(".so") {
                println!("Found plugin: {}", fname);
                let plugin = Plugin::new(fname);
                libs.push(plugin);
            }
        }

        libs
    }

    // TODO: Create plugin functions

    // Call the name() function instead of using the property
    pub fn call_name(self) -> String {
        let mut name: String = String::new();
        let get_name_func: Result<Symbol<NameProducer>, Error> = unsafe {
            self.lib.get(b"name")
        };
        match get_name_func {
            Err(err) => {
                name = self.name.clone();
                println!(
                    "Error calling name() in {}: {}",
                    self.name.clone(), err
                );
            }, Ok(get_name) => unsafe { get_name(&mut name) }
        }
        name
    }
}
