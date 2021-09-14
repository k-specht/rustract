/// Configuration requirements.
#[derive(serde::Deserialize, Debug)]
pub struct Config {
    pub db_path: String,
    pub db_type: String,
}
