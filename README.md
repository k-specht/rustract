# rustract
A Rust library for safely extracting JSON fields while also checking data bounds.

This library allows data to be quickly checked against the Database's limits to ensure that no non-required fields are missing,
all data is within the Database's byte limit, and all data follows the desired format.
The library also generates TypeScript types for the front end to use to allow for consistency in back end request data.
With the data extraction process and front end types, a web application's behavior on edge cases is secured.

A clear example of this library in action is the use of ISO dates in a Database.
An ISO date in the format YYYY-MM-DD may be ideal for Database storage, but JavaScript dates using the ISO String output will
include a time at the end in the format YYYY-MM-DDT00:00:00 or similar.
While adding front-end code for ensuring dates follow this format may help, a user using an API or app like Postman may input
invalid data that the Database will then accept.
To get around this, the back-end would also need boundary checking code, which can quickly become unmanageable.
Instead, a size limit and regex check in the JSON extraction process can greatly simplify all of the involved code.

# Using this library
Obtain an SQL schema from the Database being used (it should contain the table creation SQL).

Set the TypeScript output path and schema path in the config file and use it with the init() function.

Finally, use the generated Database Design to extract JSON in the extraction step of the web library you are using.
For the Warp library, these steps will need to be done inside a lazy_static block.
