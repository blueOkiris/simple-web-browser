/*
 * Author: Dylan Turner
 * Description: Entry point for the simple web browser
 */

mod app;

use app::SimpleBrowser;

fn main() {
    let app = SimpleBrowser::new();
    app.run();
}
