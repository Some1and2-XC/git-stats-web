// Mod for handling the auth front-end endpoints
// This includes structs etc

use actix_session::Session;
use actix_web::{http, web::{self, Data, Redirect}, Responder};
use argon2::{password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use log::debug;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, types::chrono::NaiveDateTime, FromRow, SqlitePool, Error};
use validator::Validate;

pub const SESSION_USER_ID_KEY: &str = "user_id";

use super::DbPool;

#[derive(Deserialize, Debug, FromRow)]
pub struct LoginFormData {
    pub email: String,
    pub password: String,
    pub remember: Option<String>,
}

#[derive(Serialize)]
pub struct SessionDetails {
    id: i64,
}

#[derive(Debug)]
enum CantAddUserError {
    EmailExists,
    UsernameExists,
}

#[derive(Debug, FromRow, Validate)]
pub struct User {
    pub id: Option<i64>,
    #[validate(email,)]
    pub email: String,
    pub username: String,
    password: String,
    pub credits: Option<i64>,
    pub date_created: Option<NaiveDateTime>,
    pub last_accessed: Option<NaiveDateTime>,
}

impl User {
    /// Gets a user from `SignupFormData`.
    /// Panics:
    ///  - If password and password2 don't match (this should be done before calling this function)
    ///  - If the `hash_password()` method fails (for whatever reason)
    fn from_signup_form_data(data: &SignupFormData) -> Self {

        assert_eq!(&data.password, &data.password2);

        let hasher = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);

        let password = hasher.hash_password(&data.password.as_ref(), &salt).unwrap();

        Self {
            id: None,
            email: data.email.clone(),
            username: data.username.clone(),
            password: password.to_string(),
            credits: None,
            date_created: None,
            last_accessed: None,
        }
    }

    /// Method for updating the fields of a user.
    /// This is helpful for ensuring `Nones` don't exist.
    pub async fn update_from_db_email(self, db: DbPool) -> Result<Self, Error> {
        return Ok(Self::from_db_email(&self.email, db).await?);
    }

    /// Method for getting a user from an email
    pub async fn from_db_email(email: &str, db: DbPool) -> Result<Self, Error> {
        return Ok(query_as::<_, User>("SELECT * FROM Users WHERE email=$1;")
            .bind(email)
            .fetch_one(&**db)
            .await?);
    }

    /// Method for updating the fields of a user.
    /// Gets user by ID.
    /// Panics:
    ///  - If self.id is `None`
    pub async fn update_from_db_id(self, db: DbPool) -> Result<Self, Error> {
        return Ok(Self::from_db_id(self.id.unwrap(), db).await.unwrap());
    }

    /// Returns the user from a db.
    /// Returns `None` if not found.
    pub async fn from_db_id(id: i64, db: DbPool) -> Option<Self> {
        return match query_as::<_, User>("SELECT * FROM Users WHERE id=$1;")
            .bind(id)
            .fetch_one(&**db)
            .await {
            Ok(v) => Some(v),
            Err(_) => None,
        };
    }

    /// Tries to add user to db.
    /// Returns an error if something goes wrong.
    /// Returns the updated user if not.
    async fn add_to_db(&self, db: Data<SqlitePool>) -> Result<Self, CantAddUserError> {


        if Self::username_exists(&self.username, db.clone()).await {
            return Err(CantAddUserError::UsernameExists);
        }

        if Self::email_exists(&self.email, db.clone()).await {
            return Err(CantAddUserError::EmailExists);
        }

        let result = query(
            "INSERT INTO Users (email, username, password)
            VALUES ($1, $2, $3)")
            .bind(&self.email)
            .bind(&self.username)
            .bind(&self.password)
            .execute(&**db)
            .await.unwrap();

        return Ok(
            User::from_db_id(result.last_insert_rowid(), db)
                .await.unwrap()
        );
    }

    /// Function for seeing if a username exists
    pub async fn username_exists(username: &str, db: DbPool) -> bool {
        return match query("SELECT * FROM Users WHERE username=$1;")
            .bind(username)
            .fetch_one(&**db)
        .await {
            Ok(_) => true,
            Err(_e) => false,
        };

    }

    /// Function for seeing if a email exists
    pub async fn email_exists(email: &str, db: DbPool) -> bool {
        return match query("SELECT * FROM Users WHERE email=$1;")
            .bind(email)
            .fetch_one(&**db)
        .await {
            Ok(_) => true,
            Err(_e) => false,
        };

    }

    /// Method for verifying the password of a user
    /// Panics if the password is `None`.
    fn verify_password(&self, password: &str) -> bool {

        let parsed_password = {
            PasswordHash::new(&self.password).expect("Failed to parse (probably DB) password!")
        };

        return match Argon2::default().verify_password(password.as_bytes(), &parsed_password) {
            Ok(_) => true,
            Err(_) => false,
        };
    }

    /// Returns an option containing the ID of a user according to session.
    fn get_session_id(session: &Session) -> Option<i64> {
        if let Ok(v) = session.get::<i64>(SESSION_USER_ID_KEY) {
            return v;
        } else {
            return None;
        }
    }

    /// Gets a User object from session
    // Returns None if not set
    pub async fn from_session(session: &Session, db: DbPool) -> Option<Self> {
        let id = Self::get_session_id(session)?;
        let user = match Self::from_db_id(id.clone(), db).await {
            Some(user) => user,
            None => {
                debug!("Found User ID in session but not DB, ID: #{}", id);
                return None;
            },
        };

        return Some(user);
    }

    /// Adds the user to the session
    /// Panics:
    ///  - If the user doesn't have an ID.
    ///  - If the session doesn't allow for adding users.
    fn to_session(&self, session: &Session) -> () {
        session.insert(SESSION_USER_ID_KEY, self.id.unwrap()).unwrap();
    }

    /// Updates the login date in the DB and returns self.
    /// Panics if the User doesn't have an ID
    async fn update_login_date(self, db: DbPool) -> Self {
        query("UPDATE Users SET last_accessed = CURRENT_TIMESTAMP WHERE id=$1;")
            .bind(&self.id)
            .execute(&**db)
            .await.unwrap();

        return self.update_from_db_id(db).await.unwrap();
    }
}

pub async fn login_handler(session: Session, db: DbPool, info: web::Form<LoginFormData>) -> impl Responder {

    let form_data = info.into_inner();

    let invalid_credentials: (String, http::StatusCode) = ("Invalid Credentials".into(), http::StatusCode::UNAUTHORIZED);

    let user = match User::from_db_email(&form_data.email, db.clone()).await {
        Ok(v) => v,
        Err(_e) => return invalid_credentials,
    };

    if !user.verify_password(&form_data.password) {
        return invalid_credentials;
    }

    user.to_session(&session);

    return (format!("Form Data: `{:?}`\nUser Data: `{:?}`", form_data, user.update_login_date(db).await), http::StatusCode::OK);
}

#[derive(Deserialize, Debug, FromRow)]
pub struct SignupFormData {
    pub email: String,
    pub username: String,
    pub password: String,
    pub password2: String,
    pub remember: Option<String>,
}

pub async fn signup_handler(session: Session, db: DbPool, info: web::Form<SignupFormData>) -> impl Responder {

    let form_data = info.into_inner();

    if form_data.password != form_data.password2 {
        return ("Invalid passwords (your passwords don't match)".to_string(), http::StatusCode::UNAUTHORIZED);
    }

    let user = match User::from_signup_form_data(&form_data).add_to_db(db).await {
        Ok(v) => v,
        Err(e) => {
            debug!("Failed Signup Attempt: {:?}", e);
            return (format!("Couldn't add the credentials ({:?})", e).to_string(), http::StatusCode::UNAUTHORIZED)
        },
    };

    user.to_session(&session);

    return (format!("Got some stuff: `{:?}`! User: {:?}", form_data, user), http::StatusCode::OK);
}

pub async fn logout(session: Session) -> impl Responder {
    match session.remove(SESSION_USER_ID_KEY) {
        Some(_) => debug!("Account logged out successfully!"),
        None => debug!("Account not logged out (wasn't signed in!)"),
    };

    return Redirect::to("/login");
}
