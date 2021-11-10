extern crate warp;
extern crate tokio;
extern crate lazy_static;
extern crate rustract;
use warp::Filter;
use warp::reject::Reject;
use lazy_static::lazy_static;
use std::convert::Infallible;

use rustract::db::Database;
use rustract::error::BackendError;
use rustract::init;

mod routes;

// Allows the database design to be used as a global.
// This is important because Warp's closures cannot take ownership of a non-static reference to the database.
lazy_static! {
    pub static ref DB_DESIGN: Database = init("./examples/app/example_config.json", true).expect("Failed to start example.");
}

/// Entry point into the program.
#[tokio::main]
async fn main() {
    // Lazy static will initialize it before it is used in the server (otherwise it will lag the first request to the server)
    println!("Database Initialized: {}.", !DB_DESIGN.is_empty());
    start().await.expect("Server stopped, exiting app...");
}

/// Serves the warp server on localhost, port 3030.
async fn start() -> Result<(), BackendError> {
    warp::serve(routes::get_routes().recover(handle_rejection)).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}

/// An error type enum representing the ways a client request could cause an error in the server logic.
#[derive(Debug)]
pub enum ErrorType {
    NotFound,
    Internal,
    BadRequest,
}
/// A custom error struct for making custom Warp Rejection replies.
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

/// Allows the CustomError struct to be used as a custom Warp Rejection.
impl Reject for CustomError {}

/// An example of rejection handling.
pub async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    let code;
    let message: String;

    if err.is_not_found() {
        code = warp::http::StatusCode::NOT_FOUND;
        message = "Not Found".to_string();
    } else if let Some(app_err) = err.find::<CustomError>() {
        code = app_err.to_http_status();
        message = app_err.message.to_string();
    } else if err.find::<warp::filters::body::BodyDeserializeError>().is_some() {
        code = warp::http::StatusCode::BAD_REQUEST;
        message = "Invalid Body".to_string();
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        code = warp::http::StatusCode::METHOD_NOT_ALLOWED;
        message = "Method Not Allowed".to_string();
    } else {
        // In case we missed something - log and respond with 500
        eprintln!("unhandled rejection: {:?}", err);
        code = warp::http::StatusCode::INTERNAL_SERVER_ERROR;
        message = format!("Unhandled rejection: {:?}", err);
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message,
    });

    Ok(warp::reply::with_status(json, code))
}

#[derive(serde::Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}