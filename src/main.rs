extern crate futures;
extern crate futures_cpupool;
extern crate pathfinding;
extern crate petgraph;
extern crate retry;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate sysfs_gpio;

use std::env;
use std::fs;
use std::path::Path;
use std::result;

#[macro_use]
mod gpio;

pub mod controller;

mod distance;
mod error;
mod map;
mod motor;

type Result<T> = result::Result<T, error::Error>;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Direction {
    Forward,
    Backward,
    Left,
    Right,
}

// Move commands work the following way:
// Move(Forward|Backward) => move forward or backward until next node
// Move(Left|Right) => turn 90 degrees in the given direction
#[derive(Debug, Copy, Clone)]
pub enum Command {
    Move(Direction),
    Stop,
}

fn main() {
    let mapfile = env::args().nth(1).expect("Need a link to a map file");
    let map = read_map(mapfile).unwrap();
    println!("{:?}", map);
    println!("{:#?}", map.path());
    println!("{:#?}", map.path().into_commands());
    let mut controller = controller::Builder::new()
        .front_motor_pins(2, 3, 4, 22, 17, 27)
        .rear_motor_pins(10, 9, 11, 19, 5, 6)
        .front_distance_pins(14, 15)
        .rear_distance_pins(18, 23)
        .left_distance_pins(24, 25)
        .right_distance_pins(8, 7)
        .map(map)
        .build()
        .unwrap();
    controller.run().unwrap();
}

fn read_map<P: AsRef<Path>>(path: P) -> serde_json::Result<map::Map> {
    let file = fs::File::open(path)?;
    let map = serde_json::from_reader(file)?;
    Ok(map)
}
