use futures_cpupool;
use retry;
use sysfs_gpio;

use distance;
use error::{BuilderError, Error};
use map;
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
    front_enable_a: Option<sysfs_gpio::Pin>,
    front_in_a1: Option<sysfs_gpio::Pin>,
    front_in_a2: Option<sysfs_gpio::Pin>,
    front_enable_b: Option<sysfs_gpio::Pin>,
    front_in_b1: Option<sysfs_gpio::Pin>,
    front_in_b2: Option<sysfs_gpio::Pin>,

    rear_enable_a: Option<sysfs_gpio::Pin>,
    rear_in_a1: Option<sysfs_gpio::Pin>,
    rear_in_a2: Option<sysfs_gpio::Pin>,
    rear_enable_b: Option<sysfs_gpio::Pin>,
    rear_in_b1: Option<sysfs_gpio::Pin>,
    rear_in_b2: Option<sysfs_gpio::Pin>,

    front_trigger: Option<sysfs_gpio::Pin>,
    front_echo: Option<sysfs_gpio::Pin>,

    rear_trigger: Option<sysfs_gpio::Pin>,
    rear_echo: Option<sysfs_gpio::Pin>,

    left_trigger: Option<sysfs_gpio::Pin>,
    left_echo: Option<sysfs_gpio::Pin>,

    right_trigger: Option<sysfs_gpio::Pin>,
    right_echo: Option<sysfs_gpio::Pin>,

    map: map::Map,
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
        self.front_enable_a = Some(sysfs_gpio::Pin::new(enable_a));
        self.front_in_a1 = Some(sysfs_gpio::Pin::new(in_a1));
        self.front_in_a2 = Some(sysfs_gpio::Pin::new(in_a2));
        self.front_enable_b = Some(sysfs_gpio::Pin::new(enable_b));
        self.front_in_b1 = Some(sysfs_gpio::Pin::new(in_b1));
        self.front_in_b2 = Some(sysfs_gpio::Pin::new(in_b2));
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
        self.rear_enable_a = Some(sysfs_gpio::Pin::new(enable_a));
        self.rear_in_a1 = Some(sysfs_gpio::Pin::new(in_a1));
        self.rear_in_a2 = Some(sysfs_gpio::Pin::new(in_a2));
        self.rear_enable_b = Some(sysfs_gpio::Pin::new(enable_b));
        self.rear_in_b1 = Some(sysfs_gpio::Pin::new(in_b1));
        self.rear_in_b2 = Some(sysfs_gpio::Pin::new(in_b2));
        self
    }

    pub fn front_distance_pins(mut self, trigger: u64, echo: u64) -> Self {
        self.front_trigger = Some(sysfs_gpio::Pin::new(trigger));
        self.front_echo = Some(sysfs_gpio::Pin::new(echo));
        self
    }

    pub fn rear_distance_pins(mut self, trigger: u64, echo: u64) -> Self {
        self.rear_trigger = Some(sysfs_gpio::Pin::new(trigger));
        self.rear_echo = Some(sysfs_gpio::Pin::new(echo));
        self
    }

    pub fn left_distance_pins(mut self, trigger: u64, echo: u64) -> Self {
        self.left_trigger = Some(sysfs_gpio::Pin::new(trigger));
        self.left_echo = Some(sysfs_gpio::Pin::new(echo));
        self
    }

    pub fn right_distance_pins(mut self, trigger: u64, echo: u64) -> Self {
        self.right_trigger = Some(sysfs_gpio::Pin::new(trigger));
        self.right_echo = Some(sysfs_gpio::Pin::new(echo));
        self
    }

    pub fn map(mut self, map: map::Map) -> Self {
        self.map = map;
        self
    }

    pub fn build(self) -> Result<Controller> {
        gpio_export!(self, {
            front_enable_a, front_in_a1, front_in_a2, front_enable_b, front_in_b1, front_in_b2,
            rear_enable_a, rear_in_a1, rear_in_a2, rear_enable_b, rear_in_b1, rear_in_b2,
            front_trigger, front_echo, rear_trigger, rear_echo,
            left_trigger, left_echo, right_trigger, right_echo
        });

        // Make sure export is finished
        self.poll_pin_init()?;

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
        let commands = self.map.path().into_commands();
        Ok(Controller {
            front_motors: front_motors,
            rear_motors: rear_motors,
            front_distance_sensor: front_distance_sensor,
            rear_distance_sensor: rear_distance_sensor,
            left_distance_sensor: left_distance_sensor,
            right_distance_sensor: right_distance_sensor,

            pool: futures_cpupool::CpuPool::new_num_cpus(),
            map: self.map,
            commands: commands,
        })
    }

    fn poll_pin_init(&self) -> Result<()> {
        // Unwrapping is fine here; if this fails, it means is_some is broken.
        let pins = vec![&self.front_enable_a, &self.front_in_a1, &self.front_in_a2,
                        &self.front_enable_b, &self.front_in_b1, &self.front_in_b2,
                        &self.rear_enable_a, &self.rear_in_a1, &self.rear_in_a2,
                        &self.rear_enable_b, &self.rear_in_b1, &self.rear_in_b2,
                        &self.front_trigger, &self.front_echo,
                        &self.rear_trigger, &self.rear_echo,
                        &self.left_trigger, &self.left_echo,
                        &self.right_trigger, &self.right_echo]
            .into_iter()
            .filter_map(|pin| pin.as_ref())
            .collect::<Vec<_>>();
        retry::retry(10,
                     50,
                     || {
                         pins.iter()
                             .map(|pin| pin.set_direction(pin.get_direction()?))
                             .collect::<Vec<_>>()
                     },
                     |rvec| rvec.iter().all(|res| res.is_ok()))
            .map_err(|_| Error::Build(BuilderError::ExportError))
            .and_then(|_| Ok(()))
    }
}
