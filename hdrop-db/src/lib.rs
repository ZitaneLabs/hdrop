mod schema;
pub(crate) mod database;
pub(crate) mod models;
pub use self::database::*;
pub use self::models::*;

fn main() {
    println!("Hello, world!");
}
