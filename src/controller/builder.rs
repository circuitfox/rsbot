use sysfs_gpio as gpio;

use distance;
use error::{BuilderError, Error};
use motor;
use super::Controller;
use super::super::Result;

macro_rules! build {
    ($self_: ident, $obj: ty, $err: path, {$($opt_gpio: ident),+}) => ({
        $(
            let $opt_gpio = $self_.$opt_gpio.ok_or(Error::Build($err))?;
        )+
        <$obj>::new($($opt_gpio),+)?
    });
}

// Options are used because there is no clear default for pins
#[derive(Debug, Default)]
pub struct Builder {
    front_enable_a: Option<gpio::Pin>,
    front_in_a1: Option<gpio::Pin>,
    front_in_a2: Option<gpio::Pin>,
    front_enable_b: Option<gpio::Pin>,
    front_in_b1: Option<gpio::Pin>,
    front_in_b2: Option<gpio::Pin>,

    rear_enable_a: Option<gpio::Pin>,
    rear_in_a1: Option<gpio::Pin>,
    rear_in_a2: Option<gpio::Pin>,
    rear_enable_b: Option<gpio::Pin>,
    rear_in_b1: Option<gpio::Pin>,
    rear_in_b2: Option<gpio::Pin>,

    front_trigger: Option<gpio::Pin>,
    front_echo: Option<gpio::Pin>,

    rear_trigger: Option<gpio::Pin>,
    rear_echo: Option<gpio::Pin>,

    left_trigger: Option<gpio::Pin>,
    left_echo: Option<gpio::Pin>,

    right_trigger: Option<gpio::Pin>,
    right_echo: Option<gpio::Pin>,
}

impl Builder {
    pub fn new() -> Builder {
        Builder::default()
    }

    pub fn front_motor_pins(mut self,
                            enable_a: u64,
                            in_a1: u64,
                            in_a2: u64,
                            enable_b: u64,
                            in_b1: u64,
                            in_b2: u64)
                            -> Self {
        self.front_enable_a = Some(gpio::Pin::new(enable_a));
        self.front_in_a1 = Some(gpio::Pin::new(in_a1));
        self.front_in_a2 = Some(gpio::Pin::new(in_a2));
        self.front_enable_b = Some(gpio::Pin::new(enable_b));
        self.front_in_b1 = Some(gpio::Pin::new(in_b1));
        self.front_in_b2 = Some(gpio::Pin::new(in_b2));
        self
    }

    pub fn rear_motor_pins(mut self,
                           enable_a: u64,
                           in_a1: u64,
                           in_a2: u64,
                           enable_b: u64,
                           in_b1: u64,
                           in_b2: u64)
                           -> Self {
        self.rear_enable_a = Some(gpio::Pin::new(enable_a));
        self.rear_in_a1 = Some(gpio::Pin::new(in_a1));
        self.rear_in_a2 = Some(gpio::Pin::new(in_a2));
        self.rear_enable_b = Some(gpio::Pin::new(enable_b));
        self.rear_in_b1 = Some(gpio::Pin::new(in_b1));
        self.rear_in_b2 = Some(gpio::Pin::new(in_b2));
        self
    }

    pub fn front_distance_pins(mut self, trigger: u64, echo: u64) -> Self {
        self.front_trigger = Some(gpio::Pin::new(trigger));
        self.front_echo = Some(gpio::Pin::new(echo));
        self
    }

    pub fn rear_distance_pins(mut self, trigger: u64, echo: u64) -> Self {
        self.rear_trigger = Some(gpio::Pin::new(trigger));
        self.rear_echo = Some(gpio::Pin::new(echo));
        self
    }

    pub fn left_distance_pins(mut self, trigger: u64, echo: u64) -> Self {
        self.left_trigger = Some(gpio::Pin::new(trigger));
        self.left_echo = Some(gpio::Pin::new(echo));
        self
    }

    pub fn right_distance_pins(mut self, trigger: u64, echo: u64) -> Self {
        self.right_trigger = Some(gpio::Pin::new(trigger));
        self.right_echo = Some(gpio::Pin::new(echo));
        self
    }

    pub fn build(self) -> Result<Controller> {
        gpio_export!(self, {
            front_enable_a, front_in_a1, front_in_a2, front_enable_b, front_in_b1, front_in_b2,
            rear_enable_a, rear_in_a1, rear_in_a2, rear_enable_b, rear_in_b1, rear_in_b2,
            front_trigger, front_echo, rear_trigger, rear_echo,
            left_trigger, left_echo, right_trigger, right_echo
        });

        let front_motors = build!(self, motor::Controller, BuilderError::FrontMotorPins,
                                  {front_enable_a, front_in_a1, front_in_a2,
                                   front_enable_b, front_in_b1, front_in_b2});
        let rear_motors = build!(self, motor::Controller, BuilderError::RearMotorPins,
                                 {rear_enable_a, rear_in_a1, rear_in_a2,
                                  rear_enable_b, rear_in_b1, rear_in_b2});
        let front_distance_sensor = build!(self, distance::Sensor,
                                           BuilderError::FrontDistancePins,
                                           {front_trigger, front_echo});
        let rear_distance_sensor = build!(self, distance::Sensor,
                                          BuilderError::RearDistancePins,
                                          {rear_trigger, rear_echo});
        let left_distance_sensor = build!(self, distance::Sensor,
                                          BuilderError::LeftDistancePins,
                                          {left_trigger, left_echo});
        let right_distance_sensor = build!(self, distance::Sensor,
                                           BuilderError::RightDistancePins,
                                           {right_trigger, right_echo});
        Ok(Controller {
            front_motors: front_motors,
            rear_motors: rear_motors,
            front_distance_sensor: front_distance_sensor,
            rear_distance_sensor: rear_distance_sensor,
            left_distance_sensor: left_distance_sensor,
            right_distance_sensor: right_distance_sensor,
        })
    }
}