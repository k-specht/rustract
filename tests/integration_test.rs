/* Rusty Backend
 * This test will demonstrate an example of how to use this library.
 * It also verifies that the library can be used as intended.
 * Author: Käthe Specht
 * Date: 2021-09-01
 */

use std::time::Duration;

use rustract::{error::RustractError, init};
use tokio::time::timeout;
use warp::Filter;

/// Uses the rusty backend library to generate a backend based on an example database.
#[tokio::test]
async fn main() -> Result<(), RustractError> {
    // Test this library's config integration
    let _db = init(Some("./tests/example_config.json"), None, true)?;

    // Create a future from the warp_test function
    let future = warp_test();

    // Wrap the future with a `Timeout` set to expire.
    let result = timeout(Duration::from_millis(500), future).await;

    // This seems odd, but if the warp test fails the server should have exited before the timeout
    assert!(result.is_err());

    // std::fs::remove_file("./tests/example_database.json").unwrap();

    Ok(())
}

/// Tests the warp library.
///
/// TODO: Add a client test to this that panics on failure!
async fn warp_test() -> Result<(), RustractError> {
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
