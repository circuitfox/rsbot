extern crate futures;
extern crate futures_cpupool;
extern crate pathfinding;
extern crate petgraph;
extern crate retry;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate sysfs_gpio;

use std::result;

#[macro_use]
mod gpio;

pub mod controller;

mod direction;
mod distance;
mod error;
mod map;
mod motor;

type Result<T> = result::Result<T, error::Error>;

fn main() {
    println!("Hello, world!");
}
