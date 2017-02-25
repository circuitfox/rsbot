extern crate sysfs_gpio;

use std::result;

#[macro_use]
mod gpio;
pub mod distance;
mod error;
pub mod motor;

type Result<T> = result::Result<T, error::Error>;

fn main() {
    println!("Hello, world!");
}
