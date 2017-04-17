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
    println!("Hello, world!");
    let map: map::Map = serde_json::from_value(json!({
        "nodes": [
            "false",
            "false",
            "false",
            "false",
            "false",
            "true"
        ],
        "edges": [
            {
                "nodes": [0, 1],
                "weight": "Forward",
            },
            {
                "nodes": [1, 2],
                "weight": "Left",
            },
            {
                "nodes": [2, 3],
                "weight": "Left",
            },
            {
                "nodes": [3, 4],
                "weight": "Right",
            },
            {
                "nodes": [3, 5],
                "weight": "Forward",
            }
        ]
    }))
        .unwrap();
    println!("{:?}", map);
    println!("{:#?}", map.path());
    println!("{:#?}", map.path().into_commands());
}
