pub(crate) mod database;
pub(crate) mod models;
mod schema;
mod utils;
pub use self::database::*;
pub use self::models::*;

fn main() {
    println!("Hello, world!");
}
