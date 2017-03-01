use std::thread;
use std::time;

use distance;
use motor;

use super::Result;

mod builder;

pub use self::builder::Builder;

// TODO: split movement and sensing to threads
#[derive(Debug)]
pub struct Controller {
    front_motors: motor::Controller,
    rear_motors: motor::Controller,
    front_distance_sensor: distance::Sensor,
    rear_distance_sensor: distance::Sensor,
    left_distance_sensor: distance::Sensor,
    right_distance_sensor: distance::Sensor,
}

pub enum Direction {
    Forward,
    Backward,
    Left,
    Right,
}

impl Controller {
    // Not sure about the duration parameter, may want to specialize for turning
    pub fn travel(&mut self, direction: Direction, duration: time::Duration) -> Result<()> {
        match direction {
            Direction::Forward => {
                self.front_motors.set_direction(motor::Device::A, motor::Direction::Forward)?;
                self.front_motors.set_direction(motor::Device::B, motor::Direction::Forward)?;
                self.rear_motors.set_direction(motor::Device::A, motor::Direction::Forward)?;
                self.rear_motors.set_direction(motor::Device::B, motor::Direction::Forward)?;
            }
            Direction::Backward => {
                self.front_motors.set_direction(motor::Device::A, motor::Direction::Reverse)?;
                self.front_motors.set_direction(motor::Device::B, motor::Direction::Reverse)?;
                self.rear_motors.set_direction(motor::Device::A, motor::Direction::Reverse)?;
                self.rear_motors.set_direction(motor::Device::B, motor::Direction::Reverse)?;
            }
            Direction::Left => {
                self.front_motors.set_direction(motor::Device::A, motor::Direction::Forward)?;
                self.front_motors.set_direction(motor::Device::B, motor::Direction::Reverse)?;
                self.rear_motors.set_direction(motor::Device::A, motor::Direction::Forward)?;
                self.rear_motors.set_direction(motor::Device::B, motor::Direction::Reverse)?;
            }
            Direction::Right => {
                self.front_motors.set_direction(motor::Device::A, motor::Direction::Reverse)?;
                self.front_motors.set_direction(motor::Device::B, motor::Direction::Forward)?;
                self.rear_motors.set_direction(motor::Device::A, motor::Direction::Reverse)?;
                self.rear_motors.set_direction(motor::Device::B, motor::Direction::Forward)?;
            }
        }
        self.front_motors.enable(motor::Device::A)?;
        self.front_motors.enable(motor::Device::B)?;
        self.rear_motors.enable(motor::Device::A)?;
        self.rear_motors.enable(motor::Device::B)?;

        // Let the motors move
        thread::sleep(duration);

        self.front_motors.disable(motor::Device::A)?;
        self.front_motors.disable(motor::Device::B)?;
        self.rear_motors.disable(motor::Device::A)?;
        self.rear_motors.disable(motor::Device::B)?;
        Ok(())
    }

    pub fn distance(&mut self, direction: Direction) -> Result<f32> {
        match direction {
            Direction::Forward => self.front_distance_sensor.value(),
            Direction::Backward => self.rear_distance_sensor.value(),
            Direction::Left => self.left_distance_sensor.value(),
            Direction::Right => self.right_distance_sensor.value(),
        }
    }
}
