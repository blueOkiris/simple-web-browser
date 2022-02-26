/*
 * Author: Dylan Turner
 * Description: Need to send data from main loop to gtk and reverse
 */

// For sending data from Gtk to main loop
pub enum AsyncEventType {

}

pub struct AsyncEvent {
    pub ev_type: AsyncEventType
}

impl AsyncEvent {

}

// For sending data to Gtk from main loop (requires mutex for widgets fyi)
pub struct AsyncGuiInfo {

}
