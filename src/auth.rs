// Mod for handling the auth front-end endpoints
// This includes structs etc

use actix_session::Session;
use actix_web::{http, web::{self, Redirect}, Responder};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use git_stats_web::{database::User, errors::AppError};
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};

pub const SESSION_USER_ID_KEY: &str = "user_id";

use super::DbPool;

#[derive(Serialize)]
pub struct SessionDetails {
    id: i64,
}

#[derive(Debug, FromRow, Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

impl LoginUser {
    /// Method for verifying password.
    /// Consumes self and only returns a user if the password is valid.
    pub async fn verify_password(self, pool: &Pool<Sqlite>) -> Option<User> {

        // Does checks
        let user = User::from_email(&self.email, pool).await?;
        let parsed_password = PasswordHash::new(&self.password).ok()?;
        Argon2::default().verify_password(user.password.as_bytes(), &parsed_password).ok()?;

        return Some(user);

    }

}

pub async fn login_handler(session: Session, db: DbPool, info: web::Form<LoginUser>) -> impl Responder {

    let login_user = info.into_inner();

    let invalid_credentials = Redirect::to("/login").see_other().customize();

    let user = match login_user.verify_password(&**db).await {
        Some(v) => v,
        None => return invalid_credentials,
    };

    if !user.to_session(&session) {
        warn!("You haven't been added to the session!");
    } else {
        warn!("You have been added to the session!");
    }

    // return (format!("Form Data: `{:?}`\nUser Data: `{:?}`", form_data, user.update_login_date(db).await), http::StatusCode::OK);
    return Redirect::to("/").see_other().customize();
}

#[derive(Deserialize, Debug, FromRow)]
pub struct SignupFormData {
    pub email: String,
    pub username: String,
    pub password: String,
    pub password2: String,
    pub remember: Option<String>,
}

impl SignupFormData {

    /// Tries to create a user from object.
    /// Returns error if passwords don't match.
    pub fn to_user(self) -> Result<User, AppError> {

        if self.password != self.password2 {
            return Err(AppError {
                cause: Some("Invalid passwords (your passwords don't match)".into()),
                message: Some("Invalid passwords (your passwords don't match)".into()),
                error_type: http::StatusCode::UNAUTHORIZED,
            });
        }

        println!("Here is the value of remember: {:?}", &self.remember);

        return Ok(User::new(self.email, self.username, self.password));

    }

}

pub async fn signup_handler(session: Session, db: DbPool, info: web::Form<SignupFormData>) -> impl Responder {

    let form_data = info.into_inner();

    let mut user = match form_data.to_user() {
        Ok(v) => v,
        Err(e) => return (e.message.unwrap_or("NO_MESSAGE".to_string()), e.error_type),
    };

    if user.exists(&**db).await {
        return ("Email Already Exists!".to_string(), http::StatusCode::NOT_ACCEPTABLE);
    }

    user.to_session(&session);

    // Can't wait for the .either() method for Result<T, T>
    user = match user.push_update(&**db).await {
        Ok(v) => v,
        Err(v) => v,
    };

    return (format!("Here is some data! User: `{:?}`.", user), http::StatusCode::OK);
}

pub async fn logout(session: Session) -> impl Responder {
    match session.remove(SESSION_USER_ID_KEY) {
        Some(_) => debug!("Account logged out successfully!"),
        None => debug!("Account not logged out (wasn't signed in!)"),
    };

    return Redirect::to("/login");
}
