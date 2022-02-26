/*
 * Author: Dylan Turner
 * Description:
 *  - We need to manage state globally due to nature of plugins
 *  - The "right way" to do this is to group it all into a struct that handles
 *    everything
 */

use webkit2gtk::{
    WebView, LoadEvent,
    Error,
    traits::WebViewExt
};

const BAD_URL_MSG: &'static str = "The URL can’t be shown";
const NO_INTERNET_MSG: &'static str = "Temporary failure in name resolution";

pub static mut WEB_VIEW_MANAGER: WebViewManager = WebViewManager {
    web_view: None,
    history: Vec::new(),
    curr_page: 0,
    internal_navigation: false
};

pub struct WebViewManager {
    pub web_view: Option<WebView>,
    pub history: Vec<String>,
    pub curr_page: usize,
    pub internal_navigation: bool
}

impl WebViewManager {
    pub fn new(web_view: WebView) -> Self {
        web_view.connect_load_changed(Self::web_view_load_change);
        web_view.connect_load_failed(Self::web_view_load_failed);

        Self {
            web_view: Some(web_view),
            history: Vec::new(),
            curr_page: 0,
            internal_navigation: false
        }
    }

    // Static handler for page change. Unsafe but only used internally here
    fn web_view_load_change(view: &WebView, load_ev: LoadEvent) {
        match load_ev {
            LoadEvent::Committed => {
                let uri = WebView::uri(view).unwrap().to_string();
                unsafe {
                    WEB_VIEW_MANAGER.internal_web_view_load_change(&uri);
                }
            }, _ => {}
        }
    }

    // Try searching first and then fail on second try
    fn web_view_load_failed(
            view: &WebView, _load_ev: LoadEvent, uri: &str,
            err: &Error) -> bool {
        if err.message().contains(BAD_URL_MSG) { // Try searching instead
            view.load_uri(&(String::from("https://duckduckgo.com/?q=") + uri));
            return true;
        } else if err.message().contains(NO_INTERNET_MSG) {
            // TODO: Make a no internet page
        }

        // Otherwise let it handle itself and hope nothing glitches out
        false
    }

    // Helper for web_view_load_change
    pub fn internal_web_view_load_change(&mut self, uri: &String) {
        // Changed from clicking on a web page
        if !self.internal_navigation {
            while self.curr_page < self.history.len() {
                self.history.pop();
            }

            self.history.push(uri.clone());
            self.curr_page += 1;
        } else { // Change handled by internal navigation functions
            self.internal_navigation = false;
        }
    }

    pub fn navigate_back(&mut self) {
        if self.curr_page > 1 { // Indexed by 1!!!! 0 cp === -1 hist
            let prev_page = &self.history[self.curr_page - 2];
            self.curr_page -= 1;
            self.internal_navigation = true;
            self.web_view.clone().unwrap().load_uri(prev_page);
        }
    }

    pub fn navigate_forward(&mut self) {
        if self.curr_page < self.history.len() {
            let next_page = &self.history[self.curr_page];
            self.curr_page += 1;
            self.internal_navigation = true;
            self.web_view.clone().unwrap().load_uri(next_page);
        }
    }

    pub fn navigate(&mut self, url: &String) {
        while self.curr_page < self.history.len() {
            self.history.pop();
        }

        self.history.push(url.clone());
        self.curr_page += 1;
        self.internal_navigation = true;
        self.web_view.clone().unwrap().load_uri(url);
    }
}
