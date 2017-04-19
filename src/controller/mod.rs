use std::fmt;
use std::thread;
use std::time;

use futures::future;
use futures::Future;
use futures_cpupool as cpupool;

use {Command, Direction};
use distance;
use error;
use map;
use motor;

mod builder;

const FB_THRESHOLD: f32 = 35.75;
const LR_THRESHOLD: f32 = 43.0;

pub use self::builder::Builder;

enum ThresholdLimit {
    LessThan,
    GreaterThan,
    Either,
}

pub struct Controller {
    front_motors: motor::Controller,
    rear_motors: motor::Controller,
    front_distance_sensor: distance::Sensor,
    rear_distance_sensor: distance::Sensor,
    left_distance_sensor: distance::Sensor,
    right_distance_sensor: distance::Sensor,

    pool: cpupool::CpuPool,
    map: map::Map,
    commands: Vec<Command>,
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
    pub fn run(&mut self) -> Result<(), error::Error> {
        for command in self.commands.clone() {
            match command {
                Command::Move(d) => self.travel(d).wait()?,
                Command::Stop => {
                    self.front_motors.disable(motor::Device::A)?;
                    self.front_motors.disable(motor::Device::B)?;
                    self.rear_motors.disable(motor::Device::A)?;
                    self.rear_motors.disable(motor::Device::B)?;
                }
            }
        }
        Ok(())
    }

    pub fn travel(&mut self, direction: Direction) -> cpupool::CpuFuture<(), error::Error> {
        let front_motors = self.front_motors.clone();
        let rear_motors = self.rear_motors.clone();
        // We use a sensor to indicate whether or not to stop moving
        // A sensor changes when it goes from "clear" (greater than threshold)
        // to "blocked" (less than threshold) or vice-versa.
        //
        // When turning, we care about the opposite direction from the turn direction,
        // because that sensor will change from "clear" to "blocked".
        let sensor = match direction {
            Direction::Forward => self.front_distance_sensor.clone(),
            Direction::Backward => self.rear_distance_sensor.clone(),
            Direction::Left => self.right_distance_sensor.clone(),
            Direction::Right => self.left_distance_sensor.clone(),
        };
        let left_sensor = self.left_distance_sensor.clone();
        let right_sensor = self.right_distance_sensor.clone();
        let pool = self.pool.clone();
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

            match direction {
                // Simply move until we hit the threshold
                Direction::Left | Direction::Right => {
                    let left = reach_threshold(&pool,
                                               Direction::Left,
                                               ThresholdLimit::Either,
                                               left_sensor);
                    let right = reach_threshold(&pool,
                                                Direction::Right,
                                                ThresholdLimit::Either,
                                                right_sensor);
                    left.select(right)
                        .map_err(|e| e.0)
                        .wait()?;
                }
                // This one is a bit more complex: We need to keep moving until one of the
                // following is true:
                // - The front or back sensor hits its threshold
                // - The left or right sensor exceed their threshold
                // The first case means we've hit a wall and thus a new node, and the second case
                // means that a wall has opened up and represents a new node.
                d @ Direction::Forward |
                d @ Direction::Backward => {
                    let primary = reach_threshold(&pool, d, ThresholdLimit::LessThan, sensor);
                    let left = reach_threshold(&pool,
                                               Direction::Left,
                                               ThresholdLimit::GreaterThan,
                                               left_sensor);
                    let right = reach_threshold(&pool,
                                                Direction::Right,
                                                ThresholdLimit::GreaterThan,
                                                right_sensor);
                    // Select2 will wait for either one of the futures in select to finish, or for
                    // right to finish.
                    primary.select(left)
                        .select2(right)
                        .map_err(|e| match e {
                            // type of A is (error::Error, SelectNext)
                            // type of B is error::Error
                            future::Either::A((e, _)) => e.0,
                            future::Either::B((e, _)) => e,
                        })
                        .wait()?;
                }
            }

            front_motors.disable(motor::Device::A)?;
            front_motors.disable(motor::Device::B)?;
            rear_motors.disable(motor::Device::A)?;
            rear_motors.disable(motor::Device::B)?;

            Ok(())
        })
    }

    // Not sure about the duration parameter, may want to specialize for turning
    pub fn travel_for(&mut self,
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

// Loop until the sensor value goes below its threshold
fn reach_threshold(pool: &cpupool::CpuPool,
                   direction: Direction,
                   limit: ThresholdLimit,
                   sensor: distance::Sensor)
                   -> cpupool::CpuFuture<(), error::Error> {

    let threshold = match direction {
        Direction::Forward | Direction::Backward => FB_THRESHOLD,
        Direction::Left | Direction::Right => LR_THRESHOLD,
    };
    pool.spawn_fn(move || {
        // A limit less than the threshold means that we want to avoid going below
        // the threshold. A limit greater than the threshold means we want to avoid
        // going above the threshold.
        match limit {
            ThresholdLimit::LessThan => while sensor.value()? > threshold {},
            ThresholdLimit::GreaterThan => while sensor.value()? < threshold {},
            ThresholdLimit::Either => {
                let value = sensor.value()?;
                if value > threshold {
                    loop {
                        let v = sensor.value()?;
                        if v <= threshold { break; }
                    }
                } else if value < threshold {
                    loop {
                        let v = sensor.value()?;
                        if v >= threshold { break; }
                    }
                } else {
                    // We're at the threshold, alright!
                }
            }
        }
        Ok(())
    })
}
