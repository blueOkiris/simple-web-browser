/*
 * Author: Dylan Turner
 * Description:
 * - Entry point for database forwarding server
 * - End users != database users. We want to authenticate to do ANYTHING on the DB
 *   so we have a server to ask for the db info and IT accesses the db.
 *   This is that server
 */

mod db;

use rocket::{
    routes, build, get, launch, Config,
    Request, Response,
    config::{LogLevel, TlsConfig},
    http::Header,
    fairing::{
        Fairing, Info, Kind
    }
};
use clap::{
    ArgMatches, command, arg
};
use crate::db::{
    register, login, request_password_reset, change_password,
    get_bookmark_collection, replace_bookmark_collection
};

const PORT: u16 = 9420;

static mut DB_USER: String = String::new();
static mut DB_PWORD: String = String::new();

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[launch]
fn rocket() -> _ {
    let args = get_args();
    unsafe {
        DB_USER = args.get_one::<String>("USERNAME").unwrap().clone();
        DB_PWORD = args.get_one::<String>("PASSWORD").unwrap().clone();
    }

    let mut conf = Config::release_default();
    conf.port = PORT;
    conf.log_level = LogLevel::Debug;
    conf.address = "0.0.0.0".parse().unwrap();
    conf.tls = Some(TlsConfig::from_paths(
        "/etc/ssl/ca_bundle.crt", "/etc/ssl/private/private.key"
    ));

    build().configure(conf).mount(
        "/", routes![
            register_user,
            login_user,
            request_reset_user_password,
            change_user_password,
            get_user_bookmarks//,
            //set_user_bookmarks
        ]
    ).attach(CORS)
}

fn get_args() -> ArgMatches {
    command!()
        .about("Database server for simple web browser.")
        .arg(
            arg!(<USERNAME> "Server admin username")
                .required(true)
        ).arg(
            arg!(<PASSWORD> "Server admin password")
                .required(true)
        ).get_matches()
}

#[get("/register/<email>/<password>")]
async fn register_user(email: &str, password: &str) -> String {
    let (db_user, db_pword) = unsafe {
        (DB_USER.as_str(), DB_PWORD.as_str())
    };
    let result = register(email, password, db_user, db_pword).await;
    match result {
        Ok(_) => {
            String::from("success")
        }, Err(err) => {
            println!("Failed to register user with email {}. Error: {}", email, err.to_string());
            err.to_string()
        }
    }
}

#[get("/login/<email>/<password>")]
async fn login_user(email: &str, password: &str) -> String {
    let (db_user, db_pword) = unsafe {
        (DB_USER.as_str(), DB_PWORD.as_str())
    };
    let result = login(email, password, db_user, db_pword).await;
    match result {
        Ok(_) => {
            String::from("success")
        }, Err(err) => {
            println!("Failed to register user with email {}. Error: {}", email, err.to_string());
            err.to_string()
        }
    }
}

#[get("/req_pass_rst/<email>")]
async fn request_reset_user_password(email: &str) -> String {
    let (db_user, db_pword) = unsafe {
        (DB_USER.as_str(), DB_PWORD.as_str())
    };
    let result = request_password_reset(email, db_user, db_pword).await;
    match result {
        Ok(_) => {
            String::from("success")
        }, Err(err) => {
            println!(
                "Failed to send reset for user with email {}. Error: {}", email, err.to_string()
            );
            err.to_string()
        }
    }
}

#[get("/change_pass/<email>/<code>/<new_pass>")]
async fn change_user_password(email: &str, code: &str, new_pass: &str) -> String {
    let (db_user, db_pword) = unsafe {
        (DB_USER.as_str(), DB_PWORD.as_str())
    };
    let result = change_password(email, code, new_pass, db_user, db_pword).await;
    match result {
        Ok(_) => {
            String::from("success")
        }, Err(err) => {
            println!("Failed to update password for user {}. Error: {}.", email, err.to_string());
            err.to_string()
        }
    }
}

#[get("/bookmarks/<email>/<password>")]
async fn get_user_bookmarks(email: &str, password: &str) -> String {
    let (db_user, db_pword) = unsafe {
        (DB_USER.as_str(), DB_PWORD.as_str())
    };
    match login(email, password, db_user, db_pword).await {
        Ok(_) => {},
        Err(err) => {
            println!("Failed to log in user with email {}. Error: {}", email, err.to_string());
            return err.to_string();
        }
    }

    match get_bookmark_collection(email, db_user, db_pword).await {
        Ok(bm_col) => {
            String::new()
        }, Err(err) => {
            println!(
                "Failed to get bookmarks for user with email {}. Error: {}", email, err.to_string()
            );
            err.to_string()
        }
    }
}

