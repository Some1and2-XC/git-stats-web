use core::fmt;

use actix_web::{http::StatusCode, HttpRequest, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct AppError {
    pub cause: Option<String>,
    pub message: Option<String>,
    pub error_type: StatusCode,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{:?}", self);
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AppErrorResponse {
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
