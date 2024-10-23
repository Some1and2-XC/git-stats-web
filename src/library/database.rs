//! This file is for code in relation to a database storing user data

use actix_session::Session;
use chrono::NaiveDateTime;
use futures_util::future::ok;
use log::{debug, warn};
use validator::Validate;
use serde::Deserialize;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use sqlx::{prelude::FromRow, Pool, Sqlite};

/// The session signing key for this application (this might have to be randomized)
pub const SESSION_USER_ID_KEY: &str = "user_id";

/// Struct holding database connection information
pub struct Database {
    url: String,
}

/// This enum represents the different options there are for git hosting platforms
pub enum HostOptions {
    /// Variant for github
    Github,
}

impl HostOptions {

    /// Creates string from self
    /// Uses lower case representation of name of host
    pub fn to_string(self) -> String {

        return match self {
            Self::Github => "github".to_string(),
        };

    }
}

/// A struct that represents a user
#[derive(Debug, FromRow, Validate, Deserialize)]
pub struct User {
    /// This is the user id stored in the database
    /// Is unset until user is registered (database has been updated)
    pub id: Option<i64>,
    /// This is the email of the user
    /// Should not be duplicated
    pub email: String,
    /// This is the username of the user.
    /// This should also not be duplicated
    pub username: String,
    /// This is the users password.
    /// This should ONLY store the argon2 hashed passwords.
    pub password: String,
    pub credits: i64,
    date_created: Option<NaiveDateTime>,
    last_accessed: Option<NaiveDateTime>,
    // remember: bool,
}


impl User {

    /// Method for creating a new instance of a user
    pub fn new(email: String, username: String, password: String) -> Self {

        return Self {
            id: None,
            email,
            username,
            password,
            credits: 0,
            date_created: None,
            last_accessed: None,
            // remember,
        };

    }

    /// Method for updating the DB to reflect the current user struct based on email:w
    pub async fn push_update(self, pool: &Pool<Sqlite>) -> Result<Self, Self> {

        // If user does exist
        if User::does_email_exist(&self.email, pool).await {

            let res = match sqlx::query("UPDATE Users
                SET
                    username = $2,
                    password = $3,
                    credits = $4,
                    last_accessed = CURRENT_TIMESTAMP
                WHERE email = $1
                ;
                ")
                .bind(&self.email)
                .bind(&self.username)
                .bind(&self.password)
                .bind(self.credits) // implements "copy" -> no move -> no reference required
                .execute(pool)
                .await {
                Ok(v) => v,
                Err(e) => {
                    warn!("Something went wrong with the update query: `{:?}`", e);
                    return Err(self);
                },

            };

            match res.rows_affected() {
                0 => {
                    debug!("No rows affected! Doesn't the email really exist?");
                    return Err(self);
                },
                1 => (),
                _ => {
                    panic!("Multiple Rows affected?? Something has gone very wrong! Check the values of: `{:?}`!", self);
                },
            }

            // Calls unwrap because the only way for this to fail is for res.last_insert_rowid() to not
            // work
            return User::from_email(&self.email, pool)
                .await
                .ok_or(self)
                ;

        }
        // If user doesn't exist
        else {

            debug!("Adding User with email: `{:?}`", &self.email);

            let _res = match sqlx::query(
                "INSERT INTO Users (email, username, password)
                VALUES ($1, $2, $3)")
                .bind(&self.email)
                .bind(&self.username)
                .bind(&self.password)
                .execute(pool)
                .await {
                Ok(v) => v,
                Err(e) => {
                    warn!("Failed to add user: `{:?}` to database! Error: `{:?}`", &self.email, e);
                    return Err(self)
                },
            };

            return User::from_email(&self.email, pool).await.ok_or(self);

        }

    }

    /// Updates struct based on email address
    /// It does this because id is sometimes non existant.
    /// Returns self either way however
    /// On failure, returns the original instance of self.
    /// On success, reconstructs self from database data.
    pub async fn pull_update_from_email(self, pool: &Pool<Sqlite>) -> Result<Self, Self> {

        let user: User = match sqlx::query_as("SELECT * FROM Users WHERE email = $1 LIMIT 1")
            .bind(&self.email)
            .fetch_one(pool)
            .await {
            Ok(v) => v,
            Err(_) => return Err(self),
        };

        return Ok(user);

    }

    /// Method for seeing if an instance of a user exists already in the database.
    /// This method does this by checking the self.email
    pub async fn exists(&self, pool: &Pool<Sqlite>) -> bool {

        let res = match sqlx::query("SELECT * FROM Users WHERE email = $1 LIMIT 1")
            .bind(&self.email)
            .fetch_optional(pool)
            .await {
            Ok(v) => v,
            Err(e) => {
                warn!("Failed to execute query with error: `{}`", e);
                return false;
            },
        };

        return res.is_some();

    }

    /// Returns true if the email from the user struct does exists
    /// Returns false if not
    pub async fn does_email_exist(email: &str, pool: &Pool<Sqlite>) -> bool {

        let res = match sqlx::query("SELECT * FROM Users WHERE email = $1 LIMIT 1")
            .bind(email)
            .fetch_optional(pool)
            .await {
            Ok(v) => v,
            Err(e) => {
                warn!("Failed to execute query with error: `{}`", e);

                return false;
            },
        };

        return res.is_some(); // if we found something

    }

    /// Queries the database to get an instance of a user by id
    pub async fn from_id(id: i64, pool: &Pool<Sqlite>) -> Option<Self> {

        let user: User = match sqlx::query_as("SELECT * FROM Users WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
            {
            Ok(e) => e,
            Err(e) => {
                warn!("Error: `{:?}`, failing to get user by id: `{}`", e, id);
                return None;
            },
        };

        return Some(user);

    }

    /// Queries the database to get an instance of a user by email
    pub async fn from_email(email: &str, pool: &Pool<Sqlite>) -> Option<Self> {

        let user: User = match sqlx::query_as("SELECT * FROM Users WHERE email = $1 LIMIT 1")
            .bind(email)
            .fetch_one(pool)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                warn!("Failed to get user by email: `{:?}` with error: `{:?}`", email, e);
                return None;
            },
        };

        return Some(user);

    }

    /// Method for getting the id of a user.
    /// Queries from the user email.
    /// Basically a thin shell over `get_id_from_db_email()`
    pub async fn get_id_from_db(self, pool: &Pool<Sqlite>) -> Option<i64> {

        return User::get_id_from_db_email(&self.email, pool).await;

    }

    /// Method for getting the id of a user
    /// Queries from the user email
    pub async fn get_id_from_db_email(email: &str, pool: &Pool<Sqlite>) -> Option<i64> {

        let user: User = sqlx::query_as("SELECT * FROM Users WHERE email = $1 LIMIT 1")
            .bind(email)
            .fetch_one(pool)
            .await
            .ok()?
            ;

        return user.id;

    }

    /// Adds the user to the session
    /// Returns false:
    ///  - If the user doesn't have an ID.
    ///  - If the session doesn't allow for adding users.
    pub fn to_session(&self, session: &Session) -> bool {
        if let Some(id) = self.id {
            return session.insert(SESSION_USER_ID_KEY, id).is_ok();
        } else {
            return false;
        }
    }

    /// Gets the user from the session
    pub async fn from_session(session: &Session, pool: &Pool<Sqlite>) -> Option<User> {

        let id: i64 = session.get(SESSION_USER_ID_KEY).ok()??;
        return Self::from_id(id, pool).await;

    }

}

pub struct Hosts {
    id: i64,
    host_name: String,
}

pub struct Repos {
    id: i64,
    owner_fk: i64,
    host_fk: i64,
    repo_name: String,
}

pub struct Commits {
    id: i64,
    repo_fk: i64,
    user_fk: i64,
    committer_username: String,
    committer_email: String,
    timestamp: i64,
    date_created: NaiveDateTime,
    last_modified: NaiveDateTime,
    projected_start: NaiveDateTime,
    set_start: NaiveDateTime,
}

pub struct Reports {
    id: i64,
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
    date_added: NaiveDateTime,
    date_modified: NaiveDateTime,
    repo_fk: i64,
}

pub struct RepoPermissions {
    repo_fk: i64,
    user_fk: i64,
    read: bool,
    write: bool,
    execute: bool,
}

pub struct ReportPermissions {
    report_fk: i64,
    user_fk: i64,
    read: bool,
    write: bool,
    execute: bool,
}
