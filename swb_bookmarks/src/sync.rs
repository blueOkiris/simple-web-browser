/*
 * Author: Dylan Turner
 * Description: State and functions for logging in or already being logged in
 */

use std::{
    hash::{
        Hash, Hasher
    }, collections::hash_map::DefaultHasher,
    error::Error
};
use mongodb::{
    Client, Collection,
    bson::doc
};
use serde::{
    Serialize, Deserialize
};
use rand::{
    Rng, thread_rng
};
use tokio::runtime::Runtime;
use reqwest::get;
use crate::config::Config;

const DB_LOGIN: &'static str =
    "mongodb://simple_web_browser:password@blueokiris.com:27017";
const SALT_LEN: usize = 64;

#[derive(Serialize, Deserialize)]
struct User {
    email: String,
    pword_salt: String,
    pword_hash: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Bookmark {
    pub name: String,
    pub url: String,
    pub folder: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct BookmarkCollection {
    pub user_email: String,
    pub bookmarks: Vec<Bookmark>
}

// Requires sync! Do check beforehand
pub fn get_bookmarks() -> Result<BookmarkCollection, Box<dyn Error>> {
    let cfg = Config::get_global();
    if !cfg.logged_in {
        Err("Can't sync bookmarks! Not logged in.")?
    }

    let runtime = Runtime::new().unwrap();
    let fut = get_bookmarks_async(&cfg.email);
    runtime.block_on(fut)
}

async fn get_bookmarks_async(
        email: &String) -> Result<BookmarkCollection, Box<dyn Error>> {
    let client = Client::with_uri_str(DB_LOGIN).await?;
    let bm_db = client.database("bookmarks");
    let all_bms: Collection<BookmarkCollection> = bm_db.collection("bookmarks");

    let filter = doc! {
        "user_email": email
    };
    let result = all_bms.find_one(filter, None).await?;
    
    let user_bms = match result {
        Some(ub) => ub,
        None => Err("Failed to get bookmarks.")?
    };

    Ok(user_bms)
}

pub fn register(
        email_txt: &String,
        pword_txt: &String) -> Result<(), Box<dyn Error>> {
    // Mongo is async, but we don't want that, so we wait for it to finish
    let runtime = Runtime::new().unwrap();
    let fut = register_async(email_txt, pword_txt);
    runtime.block_on(fut)
}

pub fn login(
        email_txt: &String,
        pword_txt: &String) -> Result<(), Box<dyn Error>> {
    let runtime = Runtime::new().unwrap();
    let fut = login_async(email_txt, pword_txt);
    runtime.block_on(fut)
}

async fn login_async(
        email_txt: &String,
        pword_txt: &String) -> Result<(), Box<dyn Error>> {
    let client = Client::with_uri_str(DB_LOGIN).await?;
    let bm_db = client.database("bookmarks");
    let users: Collection<User> = bm_db.collection("users");

    let filter = doc! {
        "email": email_txt
    };
    let result = users.find_one(filter, None).await?;

    let user = match result {
        Some(ref u) => u,

        // Give generic answer for security!
        None => Err("Incorrect email/password combo.")?
    };

    let salt_and_pass = user.pword_salt.clone() + pword_txt;
    let test_hash = get_hash(&salt_and_pass);

    if test_hash == user.pword_hash {
        Ok(())
    } else {
        Err("Incorrect email/password combo.")?
    }
}

async fn register_async(
        email_txt: &String,
        pword_txt: &String) -> Result<(), Box<dyn Error>> {
    assert_valid_email(email_txt).await?;

    let client = Client::with_uri_str(DB_LOGIN).await?;
    let bm_db = client.database("bookmarks");
    let users: Collection<User> = bm_db.collection("users");

    assert_unused_email(email_txt, &users).await?;

    let salt = get_random_salt();
    let salt_and_pass = salt.clone() + pword_txt;
    let hash = get_hash(&salt_and_pass.to_string());

    let new_user = User {
        email: email_txt.clone(),
        pword_salt: salt,
        pword_hash: hash
    };
    users.insert_one(new_user, None).await?;

    Ok(())
}

// Check that email is real
async fn assert_valid_email(
        email_txt: &String) -> Result<(), Box<dyn Error>> {
    let request_url = format!(
        "https://isitarealemail.com/api/email/validate?email={}",
        email_txt
    );
    let result = get(&request_url).await?.text().await?;

    if result == String::from("{\"status\":\"valid\"}") {
        Ok(())
    } else {
        Err(format!("Email '{}' is invalid.", email_txt))?
    }
}

// Check that the user doesn't already have an account
async fn assert_unused_email(
        email_txt: &String,
        users: &Collection<User>) -> Result<(), Box<dyn Error>> {
    let filter = doc!{
        "email": email_txt
    };
    let result = users.find_one(filter, None).await?;
    match result {
        Some(_err) => Err(
            format!("Email '{}' already in use.", email_txt).to_owned()
        )?, None => Ok(())
    }
}

fn get_hash(msg: &String) -> String {
    let mut hasher = DefaultHasher::new();
    msg.hash(&mut hasher);
    let num = hasher.finish();
    format!("{:x}", num)
}

fn get_random_salt() -> String {
    let mut hash_str = String::new();

    let mut rng = thread_rng();
    for _n in 0..SALT_LEN {
        let digit = rng.gen_range(0..10);
        let random_str = format!("{}", digit);
        hash_str += &random_str;
    }

    hash_str
}
