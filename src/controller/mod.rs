use std::fmt;
use std::thread;
use std::time;

use futures_cpupool as cpupool;

use direction::Direction;
use distance;
use error;
use motor;

mod builder;

pub use self::builder::Builder;

pub struct Controller {
    front_motors: motor::Controller,
    rear_motors: motor::Controller,
    front_distance_sensor: distance::Sensor,
    rear_distance_sensor: distance::Sensor,
    left_distance_sensor: distance::Sensor,
    right_distance_sensor: distance::Sensor,

    pool: cpupool::CpuPool,
}

pub struct DistanceVector {
    pub distance: f32,
    pub direction: Direction,
}

impl Drop for Controller {
    fn drop(&mut self) {
        self.front_motors.unexport();
        self.rear_motors.unexport();
        self.front_distance_sensor.unexport();
        self.rear_distance_sensor.unexport();
        self.left_distance_sensor.unexport();
        self.right_distance_sensor.unexport();
    }
}

impl fmt::Debug for Controller {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Controller")
            .field("front_motors", &self.front_motors)
            .field("rear_motors", &self.rear_motors)
            .field("front_distance_sensor", &self.front_distance_sensor)
            .field("rear_distance_sensor", &self.rear_distance_sensor)
            .field("left_distance_sensor", &self.left_distance_sensor)
            .field("right_distance_sensor", &self.right_distance_sensor)
            .finish()
    }
}

impl Controller {
    // Not sure about the duration parameter, may want to specialize for turning
    pub fn travel(&mut self,
                  direction: Direction,
                  duration: time::Duration)
                  -> cpupool::CpuFuture<(), error::Error> {
        let front_motors = self.front_motors.clone();
        let rear_motors = self.rear_motors.clone();
        self.pool.spawn_fn(move || {
            match direction {
                Direction::Forward => {
                    front_motors.set_direction(Direction::Forward)?;
                    rear_motors.set_direction(Direction::Forward)?;
                }
                Direction::Backward => {
                    front_motors.set_direction(Direction::Backward)?;
                    rear_motors.set_direction(Direction::Backward)?;
                }
                Direction::Left => {
                    front_motors.set_direction(Direction::Left)?;
                    rear_motors.set_direction(Direction::Left)?;
                }
                Direction::Right => {
                    front_motors.set_direction(Direction::Right)?;
                    rear_motors.set_direction(Direction::Right)?;
                }
            }
            front_motors.enable(motor::Device::A)?;
            front_motors.enable(motor::Device::B)?;
            rear_motors.enable(motor::Device::A)?;
            rear_motors.enable(motor::Device::B)?;

            // Let the motors move
            thread::sleep(duration);

            front_motors.disable(motor::Device::A)?;
            front_motors.disable(motor::Device::B)?;
            rear_motors.disable(motor::Device::A)?;
            rear_motors.disable(motor::Device::B)?;
            Ok(())
        })
    }

    pub fn distance(&mut self,
                    direction: Direction)
                    -> cpupool::CpuFuture<DistanceVector, error::Error> {
        let sensor = match direction {
            Direction::Forward => self.front_distance_sensor.clone(),
            Direction::Backward => self.rear_distance_sensor.clone(),
            Direction::Left => self.left_distance_sensor.clone(),
            Direction::Right => self.right_distance_sensor.clone(),
        };
        self.pool.spawn_fn(move || {
            let distance = sensor.value()?;
            Ok(DistanceVector {
                distance: distance,
                direction: direction,
            })
        })
    }
}
