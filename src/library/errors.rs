use core::fmt;

use actix_web::{http::StatusCode, HttpResponse, Responder, ResponseError};
use serde::{Deserialize, Serialize};

/// AppErrors is the error handling struct for this library.
#[derive(Debug)]
pub struct AppError {
    /// This is the internal cause of the error. (Doesn't get displayed to the user)
    pub cause: Option<String>,
    /// This is the external cause of the error. (May get displayed to the user)
    pub message: Option<String>,
    /// An appropriate HTTP status code that goes with the error (even if the error isn't HTTP related.)
    pub error_type: StatusCode,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{:?}", self);
    }
}

/// This represents a wrapper for a returned response object. The inner string is created by an `AppError` struct.
#[derive(Debug, Deserialize, Serialize)]
pub struct AppErrorResponse {
    /// The associated error
    pub error: String,
}

impl ResponseError for AppError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        return self.error_type;
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        return HttpResponse::build((&self).status_code()).json(AppErrorResponse {
            error: self.message.clone().unwrap_or("NO ERROR MESSAGE PROVIDED!".to_string()),
        });
    }
}

