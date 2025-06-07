/*
 * Author: Dylan Turner
 * Description: Entry point for the simple web browser
 */

mod app;
mod plugin;

use app::App;

fn main() {
    let app = App::new();
    app.run();
}

