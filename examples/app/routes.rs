use rusty_backend::db::Database;
use warp::Rejection;
use warp::Reply;
use warp::Filter;

use crate::CustomError;

/// Returns the route tree to be served.
pub fn get_routes() -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone  {
    warp::path!("api" / "test" / ..)
        .and(hello().or(hello()))
}

// Get /hello
/// A function that returns a warp route for Hello World.
/// TODO: Database here needs to be wrapped inside an immutable thread-safe type.
fn hello (
    // db: &Database
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("hello")
        .and(warp::get())
        .and(with_json_body())
        .and_then(move |e| say_hello(e))
}

/// Creates a hello world response for warp to reply with.
async fn say_hello(
    item: serde_json::Value,
    // db: &Database
) -> Result<impl Reply, Rejection> {
    // db.get("user").unwrap().test(item.as_array().unwrap(), true);
    respond(Ok("Hello World!!".to_string()), warp::http::StatusCode::ACCEPTED)
}

/// Uses warp to respond to the client.
fn respond<T: serde::Serialize>(result: Result<T, CustomError>, status: warp::http::StatusCode) -> Result<impl Reply, Rejection> {
    match result {
        Ok(response) => 
            Ok(warp::reply::with_status(warp::reply::json(&response), status)),
        Err(err) => 
            Err(warp::reject::custom(err))
    }
}

/// Ensures that the request contains JSON within the size limit.
fn with_json_body() -> impl Filter<Extract = (serde_json::Value,), Error = Rejection> + Clone {
    
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}
