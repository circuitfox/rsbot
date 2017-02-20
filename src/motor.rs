use std::ops::Drop;

use sysfs_gpio as gpio;
use sysfs_gpio::Pin;

#[derive(Debug)]
pub struct Controller {
    enable_a: Pin,
    in_a1: Pin,
    in_a2: Pin,
    enable_b: Pin,
    in_b1: Pin,
    in_b2: Pin
}

pub enum Device {
    A,
    B
}
pub enum Direction {
    Forward,
    Reverse
}

impl Drop for Controller {
    fn drop(&mut self) {
        gpio_unexport!(self, { enable_a, in_a1, in_a2,
                               enable_b, in_b1, in_b2 });
    }
}

impl Controller {
    // consider our own error type here?
    pub fn new(enable_a: u64, in_a1: u64, in_a2: u64,
               enable_b: u64, in_b1: u64, in_b2: u64) -> gpio::Result<Controller> {
        let controller = Controller {
            enable_a: Pin::new(enable_a),
            in_a1: Pin::new(in_a1),
            in_a2: Pin::new(in_a2),
            enable_b: Pin::new(enable_b),
            in_b1: Pin::new(in_b1),
            in_b2: Pin::new(in_b2),
        };
        gpio_export!(controller, { enable_a, in_a1, in_a2, enable_b, in_b1, in_b2 });
        Ok(controller)
    }

    pub fn enable(&self, device: Device) -> gpio::Result<()> {
        match device {
            Device::A => self.enable_a.set_value(1),
            Device::B => self.enable_b.set_value(1)
        }
    }

    pub fn disable(&self, device: Device) -> gpio::Result<()> {
        match device {
            Device::A => self.enable_a.set_value(0),
            Device::B => self.enable_b.set_value(0)
        }
    }

    pub fn set_direction(&self, device: Device, direction: Direction) -> gpio::Result<()> {
        match direction {
            Direction::Forward => match device {
                Device::A => self.set_forward(Device::A),
                Device::B => self.set_forward(Device::B)
            },
            Direction::Reverse => match device {
                Device::A => self.set_reverse(Device::A),
                Device::B => self.set_reverse(Device::B)
            }
        }
    }

    fn set_forward(&self, device: Device) -> gpio::Result<()> {
        match device {
            Device::A => {
                self.in_a1.set_value(1)?;
                self.in_a2.set_value(0)?;
                Ok(())
            },
            Device::B => {
                self.in_b1.set_value(1)?;
                self.in_b2.set_value(0)?;
                Ok(())
            }
        }
    }

    fn set_reverse(&self, device: Device) -> gpio::Result<()> {
        match device {
            Device::A => {
                self.in_a1.set_value(0)?;
                self.in_a2.set_value(1)?;
                Ok(())
            },
            Device::B => {
                self.in_b1.set_value(0)?;
                self.in_b2.set_value(1)?;
                Ok(())
            }
        }
    }
}
