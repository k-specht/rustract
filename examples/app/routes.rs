use std::collections::HashMap;
use warp::Rejection;
use warp::Reply;
use warp::Filter;
use rustract::types::DataTypeValue;

use crate::ErrorType;
use crate::DB_DESIGN;
use crate::CustomError;

/// Returns the route tree to be served.
pub fn gen_routes() -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone  {
    // <domain>/api/
    warp::path!("api" / ..)
        .and(register())
}

// GET <domain>/api/test/hello
/// A function that returns a warp route for Hello World.
fn register() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!("register")
        .and(warp::post())
        .and(with_json_body())
        .and_then(extract)
        .and_then(insert)
        .and_then(say_hello)
}

/// Extracts the data from the request body and verifies it in the process.
/// 
/// TODO: This method's error handling could probably be cleaned up.
async fn extract(body: serde_json::Value) -> Result<HashMap<String, DataTypeValue>, warp::reject::Rejection> {
    // The map this function will extract from the JSON body
    let mut map: HashMap<String, DataTypeValue> = HashMap::new();

    // Checks to make sure the data exists/is structured properly
    if let Some(data_map) = body.as_object() {
        for key in DB_DESIGN.table("user").unwrap().fields.keys() {
            let field = DB_DESIGN.table("user")
                .unwrap()
                .field(key)
                .unwrap();
            if let Some(data) = data_map.get(&field.field_design_title) {
                match field.extract(data) {
                    Ok(data_value) => {
                        map.insert(
                            field.field_design_title.to_string(),
                            data_value
                        );
                    },
                    Err(error) => {
                        return Err(warp::reject::custom(CustomError {
                            err_type: ErrorType::BadRequest,
                            message: format!("field {} is not formatted properly: {}", &field.field_design_title, error.to_string())
                        }));
                    }
                }
            } else if field.required && !field.generated {
                return Err(warp::reject::custom(CustomError {
                    err_type: ErrorType::BadRequest,
                    message: format!("field {} is listed as required, but was not included in the request body", &field.field_design_title),
                }));
            }
        }
        Ok(map)
    } else {
        Err(warp::reject::custom(CustomError {
            err_type: ErrorType::BadRequest,
            message: format!("failed to parse JSON as object, JSON: \"{}\" (Err: Body should be a map)", body.to_string()),
        }))
    }
}

/// Uses the fields to create some query or handle some type of custom logic.
async fn insert(req: HashMap<String, DataTypeValue>) -> Result<String, warp::reject::Rejection> {
    // The req variable now has all the User fields as specified in the field design
    let name = match req.get("name").unwrap() {
        DataTypeValue::String(data) => data,
        _ => panic!("invalid data type retrieved")
    };
    let email = match req.get("email").unwrap() {
        DataTypeValue::String(data) => data,
        _ => panic!("invalid data type retrieved")
    };
    let date = match req.get("date").unwrap() {
        DataTypeValue::String(data) => data,
        _ => panic!("invalid data type retrieved")
    };

    // An SQL query can be made here that safely inserts the verified data
    print!("Found User: {{ name: {}, email: {}, date: {} }}", name, email, date);
    Ok(name.to_string())
}

/// Creates a hello response for warp to reply with.
async fn say_hello(user_name: String) -> Result<impl Reply, Rejection> {
    respond(
        Ok(format!(
            "Welcome, {}! If this was hooked up to a database, you would be added.",
            user_name)
        ),
        warp::http::StatusCode::ACCEPTED
    )
}

/// Uses warp to respond to the client.
/// 
/// Status is the status code on success.
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
