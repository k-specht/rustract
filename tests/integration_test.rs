/* Rusty Backend
 * This test will demonstrate an example of how to use this library.
 * It also verifies that the library can be used as intended.
 * Author: Käthe Specht
 * Date: 2021-09-01
 */

use rusty_backend::init;

/// Uses the rusty backend library to generate a backend based on an example database.
#[test]
fn main() {
    init("./tests/example_config.json").expect("Integration test failed.");
}
