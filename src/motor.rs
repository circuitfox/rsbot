use std::ops::Drop;

use sysfs_gpio as gpio;
use sysfs_gpio::Pin;

use error;
use super::Result;

#[derive(Debug)]
pub struct Controller {
    enable_a: Pin,
    in_a1: Pin,
    in_a2: Pin,
    enable_b: Pin,
    in_b1: Pin,
    in_b2: Pin,
}

pub enum Device {
    A,
    B,
}
pub enum Direction {
    Forward,
    Reverse,
}

impl Drop for Controller {
    fn drop(&mut self) {
        gpio_unexport!(self, {enable_a, in_a1, in_a2,
                              enable_b, in_b1, in_b2});
    }
}

impl Controller {
    // consider our own error type here?
    pub fn new(enable_a: Pin,
               in_a1: Pin,
               in_a2: Pin,
               enable_b: Pin,
               in_b1: Pin,
               in_b2: Pin)
               -> Result<Controller> {
        let controller = Controller {
            enable_a: enable_a,
            in_a1: in_a1,
            in_a2: in_a2,
            enable_b: enable_b,
            in_b1: in_b1,
            in_b2: in_b2,
        };
        gpio_out!(controller, {enable_a, in_a1, in_a2,
                               enable_b, in_b1, in_b2});
        Ok(controller)
    }

    pub fn enable(&self, device: Device) -> Result<()> {
        match device {
            Device::A => self.enable_a.set_value(1).map_err(error::Error::from),
            Device::B => self.enable_b.set_value(1).map_err(error::Error::from),
        }
    }

    pub fn disable(&self, device: Device) -> Result<()> {
        match device {
            Device::A => self.enable_a.set_value(0).map_err(error::Error::from),
            Device::B => self.enable_b.set_value(0).map_err(error::Error::from),
        }
    }

    pub fn set_direction(&self, device: Device, direction: Direction) -> Result<()> {
        match direction {
            Direction::Forward => {
                match device {
                    Device::A => self.set_forward(Device::A),
                    Device::B => self.set_forward(Device::B),
                }
            }
            Direction::Reverse => {
                match device {
                    Device::A => self.set_reverse(Device::A),
                    Device::B => self.set_reverse(Device::B),
                }
            }
        }
    }

    fn set_forward(&self, device: Device) -> Result<()> {
        match device {
            Device::A => {
                self.in_a1.set_value(1)?;
                self.in_a2.set_value(0)?;
                Ok(())
            }
            Device::B => {
                self.in_b1.set_value(1)?;
                self.in_b2.set_value(0)?;
                Ok(())
            }
        }
    }

    fn set_reverse(&self, device: Device) -> Result<()> {
        match device {
            Device::A => {
                self.in_a1.set_value(0)?;
                self.in_a2.set_value(1)?;
                Ok(())
            }
            Device::B => {
                self.in_b1.set_value(0)?;
                self.in_b2.set_value(1)?;
                Ok(())
            }
        }
    }
}
