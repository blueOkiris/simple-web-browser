/*
 * Author: Dylan Turner
 * Description:
 * - webkit2gtk-rs does not yet support UserContentManager::add_filter
 * - I'm implementing it using unsafe ffi that HAS been implemented already
 * - Might update when webkit2gtk-rs finally gets it
 */

use std::{
    error::Error,
    ptr::null,
    ffi::{
        c_void, CStr
    }, fs::{
        File, create_dir_all, write
    }, io::{
        Read, BufReader      
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
use dirs::config_dir;
use webkit2gtk_sys::{
    webkit_user_content_filter_store_new,
    webkit_user_content_filter_store_load,
    webkit_user_content_filter_store_load_finish,
    WebKitUserContentFilterStore, WebKitUserContentManager, webkit_user_content_manager_add_filter, webkit_user_content_filter_store_save
};
use webkit2gtk::{
    WebView, WebViewExt
};
use reqwest::get;
use tokio::runtime::Runtime;

const BLOCK_LIST_IDENT: *const i8 = "blocklist".as_ptr() as *const i8;
const DATA_URL: &'static str =
    "https://easylist-downloads.adblockplus.org/easylist_min_content_blocker.json";

pub fn add_filter(web_view: &WebView) {
    let con_man = web_view.user_content_manager();
    let mut conf = config_dir().unwrap();
    conf.push("swb");
    conf.push("adblock");
    let store_path = String::from(conf.as_os_str().to_str().unwrap()) + "/filter_store";
    let store_path = store_path.as_str().as_ptr() as *const i8;
    unsafe {
        let con_man_ptr: *mut WebKitUserContentManager = con_man.as_ref().to_glib_none().0;
        let filter_store = webkit_user_content_filter_store_new(store_path);
        webkit_user_content_filter_store_load(
            filter_store, BLOCK_LIST_IDENT, null::<GCancellable>() as *mut _,
            Some(filter_load_callback), con_man_ptr as *mut c_void
        );
    
    }
}

pub fn update_filter(web_view: &WebView) {
    let con_man = web_view.user_content_manager();

    let mut conf = config_dir().unwrap();
    conf.push("swb");
    conf.push("adblock");
    let store_path = String::from(conf.as_os_str().to_str().unwrap()) + "/filter_store";
    let store_path = store_path.as_str().as_ptr() as *const i8;

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

    unsafe {
        let fl_arr = g_bytes_new(fl_data as *const c_void, fl_buff.len());
        let filter_store = webkit_user_content_filter_store_new(store_path);
        let con_man_ptr: *mut WebKitUserContentManager = con_man.as_ref().to_glib_none().0;
        webkit_user_content_filter_store_save(
            filter_store, BLOCK_LIST_IDENT, fl_arr, null::<GCancellable>() as *mut _,
            Some(filter_save_callback), con_man_ptr as *mut _
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

async fn download_filter_list() -> Result<String, Box<dyn Error>> {
    let text = get(DATA_URL).await?.text().await?;
    Ok(text)
}

fn save_filter_list_to_file() -> Result<String, Box<dyn Error>> {
    let runtime = Runtime::new().unwrap();
    let resp = runtime.block_on(download_filter_list())?;

    let mut conf = config_dir().unwrap();
    conf.push("swb");
    conf.push("adblock");

    if !conf.clone().exists() {
        create_dir_all(conf.clone())?;
    }

    let file_name = String::from(conf.as_os_str().to_str().unwrap()) + "/easylist.txt";
    write(file_name.clone(), resp)?;

    Ok(file_name)
}

fn get_filter_list() -> Result<Vec<u8>, Box<dyn Error>> {
    let file_name = save_filter_list_to_file()?;

    let filter_list = File::open(file_name)?;
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

