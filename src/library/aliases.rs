extern crate sqlx;
extern crate actix_web;

use sqlx::SqlitePool;
use actix_web::web::Data;
use crate::{
    calendar::CalendarValue,
    prediction::PredictionAttributes,
};

/// A type alias for calendar value with additional attributes
pub type AnnotatedCalendarValue = (CalendarValue, Vec<(PredictionAttributes, i32)>);

/// Timestamp for git stuff
pub type Timestamp = core::primitive::i64;

/// Datatype for a database pool
pub type DbPool = Data<SqlitePool>;
