/*
 * Author: Dylan Turner
 * Description:
 * - webkit2gtk-rs does not yet support UserContentManager::add_filter
 * - I'm implementing it using unsafe ffi that HAS been implemented already
 * - Might update when webkit2gtk-rs finally gets it
 */

use std::{
    ptr::null,
    ffi::{
        c_void, CStr
    }, fs::File,
    io::{
        Read, BufReader, Error        
    }
};
use gtk::{
    glib::{
        object::GObject,
        ffi::{
            GError, g_bytes_new
        }, translate::ToGlibPtr
    }, gio::ffi::{
        GAsyncResult, GCancellable    
    }
};
use webkit2gtk_sys::{
    webkit_user_content_filter_store_new,
    webkit_user_content_filter_store_load,
    webkit_user_content_filter_store_load_finish,
    WebKitUserContentFilterStore, WebKitUserContentManager, webkit_user_content_manager_add_filter, webkit_user_content_filter_store_save
};
use webkit2gtk::{
    WebView, WebViewExt
};

const FILTER_FILE: *const i8 = "filter_cache".as_ptr() as *const i8;
const BLOCK_LIST_IDENT: *const i8 = "blocklist".as_ptr() as *const i8;
const DATA_FILE: &'static str = "easylist.json";

pub fn add_filter(web_view: &WebView) {
    let con_man = web_view.user_content_manager();

    unsafe {
        let con_man_ptr: *mut WebKitUserContentManager = con_man.as_ref().to_glib_none().0;
        let filter_store = webkit_user_content_filter_store_new(FILTER_FILE);
        webkit_user_content_filter_store_load(
            filter_store, BLOCK_LIST_IDENT, null::<GCancellable>() as *mut _,
            Some(filter_load_callback), con_man_ptr as *mut c_void
        );
    
    }
}

unsafe extern "C" fn filter_load_callback(
        caller: *mut GObject, res: *mut GAsyncResult, con_man_ptr: *mut c_void) {
    let filter_store = caller as *mut WebKitUserContentFilterStore;
    let mut error = null::<GError>() as *mut GError;
    let filter = webkit_user_content_filter_store_load_finish(filter_store, res, &mut error);

    if error.is_null() {
        let con_man = con_man_ptr as *mut WebKitUserContentManager;
        webkit_user_content_manager_add_filter(con_man, filter);
    } else { // We haven't saved the filter list before, so let's do that
        let real_err = *error;
        let error_msg = real_err.message;
        println!("GError Warning: {}", CStr::from_ptr(error_msg).to_str().unwrap_or(""));

        let fl_buff = get_filter_list();
        if fl_buff.is_err() {
            println!(
                "Failed to load filter list! Error: {}.\nIgnoring.",
                fl_buff.as_ref().err().unwrap().to_string()
            );
            return;
        }
        let fl_buff = fl_buff.unwrap();
        let fl_data = fl_buff.as_ptr();
        let fl_arr = g_bytes_new(fl_data as *const c_void, fl_buff.len());

        webkit_user_content_filter_store_save(
            filter_store, BLOCK_LIST_IDENT, fl_arr, null::<GCancellable>() as *mut _,
            Some(filter_save_callback), con_man_ptr
        );
    }
}

fn get_filter_list() -> Result<Vec<u8>, Error> {
    let filter_list = File::open(DATA_FILE)?;
    let mut filter_list_reader = BufReader::new(filter_list);
    let mut filter_list_buff = Vec::new();
    filter_list_reader.read_to_end(&mut filter_list_buff)?;
    Ok(filter_list_buff)
}

unsafe extern "C" fn filter_save_callback(
        caller: *mut GObject, res: *mut GAsyncResult, con_man_ptr: *mut c_void) {
    let filter_store = caller as *mut WebKitUserContentFilterStore;
    let mut error = null::<GError>() as *mut GError;
    let filter = webkit_user_content_filter_store_load_finish(filter_store, res, &mut error);

    if error.is_null() {
        let con_man = con_man_ptr as *mut WebKitUserContentManager;
        webkit_user_content_manager_add_filter(con_man, filter);
    } else { // Tried and failed. Give up
        let real_err = *error;
        let error_msg = real_err.message;
        println!("GError: {}", CStr::from_ptr(error_msg).to_str().unwrap_or(""));
        
        println!("Failed to save and load filter list :(\nNo adblock for you, sorry!");
    }
}

