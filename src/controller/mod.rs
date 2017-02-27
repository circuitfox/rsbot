use distance;
use motor;

mod builder;

pub use self::builder::Builder;

#[derive(Debug)]
pub struct Controller {
    front_motors: motor::Controller,
    rear_motors: motor::Controller,
    front_distance_sensor: distance::Sensor,
    rear_distance_sensor: distance::Sensor,
    left_distance_sensor: distance::Sensor,
    right_distance_sensor: distance::Sensor,
}
