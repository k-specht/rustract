extern crate warp;
extern crate rusty_backend;
extern crate tokio;
use rusty_backend::error::BackendError;
use rusty_backend::init;
use warp::Filter;

#[tokio::main]
async fn main() {
    let _db = init("./tests/example_config.json", true).expect("Database design initialization failed.");
    warp_test().await.expect("Server stopped, exiting app...");
}

/// Tests the warp library.
///
/// TODO: Add a client test to this that panics on failure!
async fn warp_test() -> Result<(), BackendError> {
    let hello_world = warp::path::end().map(|| "Hello, World at root!");
    let numb = warp::path!(u16).map(|a| format!("{}", a));
    let path = warp::path("hello").and(numb);
    let routes = warp::get()
        .and(hello_world
        .or(path)
    );
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}
