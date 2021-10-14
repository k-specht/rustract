extern crate warp;
extern crate rusty_backend;
extern crate tokio;
use std::convert::Infallible;

use rusty_backend::db::Database;
use rusty_backend::error::BackendError;
use rusty_backend::init;
use warp::Filter;
use warp::reject::Reject;

mod routes;

#[tokio::main]
async fn main() {
    // static db: Database = init("./tests/example_config.json", true).expect("Database design initialization failed.");
    let db: rusty_backend::db::Database = Database::from_schema("../../tests/schema.sql").unwrap();
    start().await.expect("Server stopped, exiting app...");
    print!("{}", db.is_empty());
}

/// Serves the warp server.
async fn start() -> Result<(), BackendError> {
    warp::serve(routes::get_routes().recover(handle_rejection)).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}

#[derive(Debug)]
pub enum ErrorType {
    NotFound,
    Internal,
    BadRequest,
}
#[derive(Debug)]
pub struct CustomError {
    pub err_type: ErrorType,
    pub message: String,
}

impl CustomError {
    pub fn to_http_status(&self) -> warp::http::StatusCode {
        match self.err_type {
            ErrorType::NotFound => warp::http::StatusCode::NOT_FOUND,
            ErrorType::Internal => warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ErrorType::BadRequest => warp::http::StatusCode::BAD_REQUEST,
        }
    }
}

impl Reject for CustomError {
    
}

pub async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = warp::http::StatusCode::NOT_FOUND;
        message = "Not Found";
    } else if let Some(app_err) = err.find::<CustomError>() {
        code = app_err.to_http_status();
        message = app_err.message.as_str();
    } else if err.find::<warp::filters::body::BodyDeserializeError>().is_some() {
        code = warp::http::StatusCode::BAD_REQUEST;
        message = "Invalid Body";
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        code = warp::http::StatusCode::METHOD_NOT_ALLOWED;
        message = "Method Not Allowed";
    } else {
        // In case we missed something - log and respond with 500
        eprintln!("unhandled rejection: {:?}", err);
        code = warp::http::StatusCode::INTERNAL_SERVER_ERROR;
        message = "Unhandled rejection";
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}

#[derive(serde::Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}