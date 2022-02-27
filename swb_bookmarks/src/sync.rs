/*
 * Author: Dylan Turner
 * Description: State and functions for logging in or already being logged in
 */

pub static mut LOGIN_MANAGER: LoginManager = LoginManager {
    synced: false
};

pub struct LoginManager {
    synced: bool
}

impl LoginManager {
    pub fn login(&mut self) {
        
    }
}
