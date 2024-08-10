extern crate sqlx;
extern crate actix_web;

use sqlx::SqlitePool;
use actix_web::web::Data;

/// Timestamp for git stuff
pub type Timestamp = core::primitive::i64;

/// Datatype for a database pool
pub type DbPool = Data<SqlitePool>;
