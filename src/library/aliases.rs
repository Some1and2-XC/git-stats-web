use sqlx::SqlitePool;
use actix_web::web::Data;

/// Timestamp for git stuff
pub use i64 as Timestamp;

/// Datatype for a database pool
pub type DbPool = Data<SqlitePool>;
