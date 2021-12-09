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

const DB_URL: &'static str = "mongodb://blueokiris.com:27017";

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

// Query the user's salt from the database
pub async fn query_salt(username: &String) -> Result<String, Box<dyn Error>> {
    info!("Connecting to sync server.");
    let client = Client::with_uri_str(DB_URL).await?;
    let bm_db = client.database("bookmarks");
    let users: Collection<User> = bm_db.collection("users");
    info!("Successfully connected to sync server.");

    let filter = doc! { "username": username.clone() };
    let result = users.find_one(filter, None).await?;
    match result {
        Some(ref user) => {
            info!("Retrieved user salt: {}", user.password_salt);
            Ok(user.password_salt.clone())
        }, None => Err(format!("User not found: {}", username).to_owned())?
    }
}
