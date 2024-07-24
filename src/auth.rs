// Mod for handling the auth front-end endpoints
// This includes structs etc

use actix_web::web;
use serde::Deserialize;


#[derive(Deserialize, Debug)]
pub struct LoginFormData {
    pub email: String,
    pub password: String,
    pub remember: Option<String>,
}


pub async fn login_handler(info: web::Form<LoginFormData>) -> String {
    return format!("Got some stuff! `{:?}`", info.into_inner());
}

#[derive(Deserialize, Debug)]
pub struct SignupFormData {
    pub email: String,
    pub username: String,
    pub password: String,
    pub password2: String,
    pub remember: Option<String>,
}

pub async fn signup_handler(info: web::Form<SignupFormData>) -> String {
    return format!("Got some stuff! `{:?}`", info.into_inner());
}
