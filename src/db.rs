/*
 * Author: Dylan Turner
 * Description: Handle database stuff
 */

use mongodb::{
    Client, Collection,
    bson::doc
};
use serde::{ Serialize, Deserialize };
use std::error::Error;
use log::info;

const DB_LOGIN: &'static str =
    "mongodb://simple_web_browser:password@blueokiris.com:27017";

/*
 * Users exist as the following structure:
 * {
 *     username: "username",
 *     password_salt: "salt",
 *     password_hash: "hash"
 * }
 * 
 * And then links are stored here:
 * {
 *     user: "username",
 *     links: [ // Folders level
 *         [ // Folder level
 *             [ "name", "url" ], // Bookmark level
 *             ...
 *         ],
 *         ...
 *     ]
 * }
 * 
 * Later, a passwords system may be implemented
 */

#[derive(Serialize, Deserialize)]
struct User {
    username: String,
    password_salt: String,
    password_hash: String
}

pub fn query_salt(username: &String) -> Result<String, String> {
    Ok(String::new())
}
