# rustract [![Latest Version](https://img.shields.io/crates/v/rustract.svg)](https://crates.io/crates/rustract)
A Rust library for safely extracting JSON fields while also checking data bounds.

# Installation:
```
[dependencies]
rustract = "0.1.0"
```

# Initialization
Obtain an SQL schema from the Database being used (it should contain the table creation SQL).
Alternatively, one of the example database design outputs can be copied and modified for smaller projects.
In this case, the config file must point to this database design copy using the `db_path` field.

Create a config file (JSON) that contains a `schema_path` field which is a path to the schema.
Optionally, set a path for the TypeScript files to be placed into using the `type_path` field.

Finally, the database design can be manually loaded in using the library's functions, 
or initialized using the `init(config_path)` function.

# Usage
Each database design has table designs that are composed of field designs.
The main way to extract and test data from JSON data is by using the `extract(json)` function of the `FieldDesign` struct.
This will extract one value from the provided JSON object.

If an entire table is being extracted from the JSON data, the `TableDesign` struct also has an `extract(json)` method.
This method calls each contained field's `extract(json)` method.

The requirements of each field are mostly read from the schema, 
but the output `DatabaseDesign` file should *always* be manually checked and edited for accuracy.
Specifically, regular expressions and size requirements for each field will usually need to be added manually.

For some libraries, like Warp, the `DatabaseDesign` struct must be static.
It is recommended to use the `lazy_static` library to accomplish this.

# Methodology
This library allows data to be quickly checked against the Database's limits to ensure that zero required fields are missing,
all data is within the Database's byte and length limits, and all data follows the desired format.
The library also generates TypeScript types for the front end, to allow for consistency in back-end request data.
With the data extraction process and front end types, a web application's behavior on edge cases is secured.

A clear example of this library in action is the use of ISO dates in a Database.
An ISO date in the format YYYY-MM-DD may be ideal for Database storage, but JavaScript dates using the ISO String output will
include a time at the end, usually in the format YYYY-MM-DDT00:00:00.
While adding front-end code for ensuring dates follow this format may help, a user using an API or app like Postman may input
invalid data that the Database will then accept.

To get around this, the back end would also need boundary checking code, which can quickly become unmanageable.
Instead, a size limit and regex check in the JSON extraction process can greatly simplify all of the involved code.
It also creates consistency in the Database fields.
