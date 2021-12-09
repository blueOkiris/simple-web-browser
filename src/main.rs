/*
 * Author: Dylan Turner
 * Description: Entry point for the simple web browser
 */

mod gui;
mod app;
mod db;

use log4rs::init_file;
use app::start_browser;

fn main() {
    init_file("logging_config.yaml", Default::default()).unwrap();
    start_browser();
}
