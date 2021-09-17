use crate::types::Config;
use sqlx::mysql::MySqlPoolOptions;

pub fn init(config: Config) {
    print!("Stub: {}", config.db_type);
    match config.type_path {
        Some(path) => print!("Path: {}", path),
        None => panic!("No path found!"),
    }
}
