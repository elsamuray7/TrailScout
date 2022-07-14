use std::fmt::{Display, Formatter};
use actix_web::{error, get, http::{header::ContentType, StatusCode}, App, HttpResponse, ResponseError};
use derive_more::{Display, Error};

///Custom Error for TrailScout
#[derive(Debug, Display, Error)]
pub enum TailScoutError {
    #[display(fmt = "{}", message)]
    InternalError {
        message:String
    },

    #[display(fmt = "{}", message)]
    BadClientData {
        message:String
    },
}


impl ResponseError for TailScoutError {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::InternalError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::BadClientData { .. } => StatusCode::BAD_REQUEST,
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(self.to_string())
    }
}