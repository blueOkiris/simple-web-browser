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



