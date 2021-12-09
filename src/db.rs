/*
 * Author: Dylan Turner
 * Description: Handle database stuff
 */

use mongodb::{
    Client, Collection,
    bson::doc
};
use std::{
    hash::{ Hash, Hasher },
    collections::hash_map::DefaultHasher,
    error::Error
};
use log::info;
use serde::{ Serialize, Deserialize };
use reqwest::get;
use rand::{ Rng, thread_rng };
use tokio::runtime::Runtime;

const DB_LOGIN: &'static str =
    "mongodb://simple_web_browser:password@blueokiris.com:27017";
const SALT_LEN: usize = 64;

#[derive(Serialize, Deserialize)]
struct User {
    email: String,
    password_salt: String,
    password_hash: String
}

// Prevent spam by enforcing real emails when registering
async fn assert_valid_email(email: &String) -> Result<(), Box<dyn Error>> {
    let request_url = format!(
        "https://isitarealemail.com/api/email/validate?email={}",
        email
    );
    let result = get(&request_url).await?.text().await?;
    info!("Email result: {}", result);
    
    if result == String::from("{\"status\":\"valid\"}") {
        Ok(())
    } else {
        Err(format!("Email '{}' is invalid.", email).to_owned())?
    }
}

// Make sure we're not currently using the email
async fn assert_unused_email(
        email: &String,
        users: &Collection<User>) -> Result<(), Box<dyn Error>> {
    
    let filter = doc! { "email": email };
    let result = users.find_one(filter, None).await?;
    match result {
        Some(_) => Err(
            format!("Email '{}' already in use.", email).to_owned()
        )?,
        None => Ok(())
    }
}

fn get_random_hash() -> String {
    let mut ret = String::new();
    let mut rng = thread_rng();
    for _ in 0..SALT_LEN {
        let digit = rng.gen_range(0..10);
        let random_str = format!("{}", digit);
        ret += &random_str;
    }
    ret
}

fn get_hash(msg: &String) -> String {
    let mut hasher = DefaultHasher::new();
    msg.hash(&mut hasher);
    let num = hasher.finish();
    format!("{:x}", num)
}

async fn login_async(
        email: &String, password: &String) -> Result<(), Box<dyn Error>> {
    let client = Client::with_uri_str(DB_LOGIN).await?;
    let bookmark_db = client.database("bookmarks");
    let users: Collection<User> = bookmark_db.collection("users");

    let filter = doc! { "email": email };
    let result = users.find_one(filter, None).await?;

    let user = match result {
        Some(ref u) => u,

        // Give generic answer here for security
        None => Err(format!("Incorrect email/password combo.").to_owned())?
    };

    // Calculate what hash would be give user salt
    let salt_and_pass = user.password_salt.clone() + password;
    let test_hash = get_hash(&salt_and_pass);

    if test_hash == user.password_hash {
        Ok(())
    } else {
        Err(format!("Incorrect email/password combo.").to_owned())?
    }
}

pub fn login(
        email: &String, password: &String) -> Result<(), Box<dyn Error>> {
    let runtime = Runtime::new().unwrap();
    let fut = login_async(email, password);
    runtime.block_on(fut)
}

async fn register_async(
        email: &String, password: &String) -> Result<(), Box<dyn Error>> {
    assert_valid_email(&email).await?;

    let client = Client::with_uri_str(DB_LOGIN).await?;
    let bookmark_db = client.database("bookmarks");
    let users: Collection<User> = bookmark_db.collection("users");
    
    assert_unused_email(&email, &users).await?;

    let salt = get_random_hash();
    let salt_and_pass = salt.clone() + password;
    let hash = get_hash(&salt_and_pass);

    let new_user = User {
        email: email.clone(),
        password_salt: salt,
        password_hash: hash
    };
    users.insert_one(new_user, None).await?;

    Ok(())
}

pub fn register(
        email: &String, password: &String) -> Result<(), Box<dyn Error>> {
    let runtime = Runtime::new().unwrap();
    let fut = register_async(email, password);
    runtime.block_on(fut)
}
