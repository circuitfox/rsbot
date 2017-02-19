use sysfs_gpio as gpio;
use sysfs_gpio::Pin;

macro_rules! gpio_out {
    ($gpio:ident, $pin: expr) => (
        let $gpio = Pin::new($pin);
        $gpio.with_exported(|| {
            $gpio.set_direction(gpio::Direction::Out)?;
            Ok(())
        })?;
    )
}

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

impl Controller {
    // consider our own error type here?
    pub fn new(enable_a: u64, in_a1: u64, in_a2: u64,
               enable_b: u64, in_b1: u64, in_b2: u64) -> gpio::Result<Controller> {
        gpio_out!(enable_a, enable_a);
        gpio_out!(in_a1, in_a1);
        gpio_out!(in_a2, in_a2);
        gpio_out!(enable_b, enable_b);
        gpio_out!(in_b1, in_b1);
        gpio_out!(in_b2, in_b2);
        Ok(Controller {
            enable_a: enable_a,
            in_a1: in_a1,
            in_a2: in_a2,
            enable_b: enable_b,
            in_b1: in_b1,
            in_b2: in_b2,
        })
    }

    pub fn enable(&self, device: Device) -> gpio::Result<()> {
        match device {
            Device::A => self.enable_a.with_exported(|| {
                self.enable_a.set_value(1)?;
                Ok(())
            }),
            Device::B => self.enable_b.with_exported(|| {
                self.enable_b.set_value(1)?;
                Ok(())
            })
        }
    }

    pub fn disable(&self, device: Device) -> gpio::Result<()> {
        match device {
            Device::A => self.enable_a.with_exported(|| {
                self.enable_a.set_value(0)?;
                Ok(())
            }),
            Device::B => self.enable_b.with_exported(|| {
                self.enable_b.set_value(0)?;
                Ok(())
            })
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
                let res_a1 = self.in_a1.with_exported(|| {
                    self.in_a1.set_value(1)?;
                    Ok(())
                });
                let res_a2 = self.in_a2.with_exported(|| {
                    self.in_a2.set_value(0)?;
                    Ok(())
                });
                res_a1.and(res_a2)
            },
            Device::B => {
                let res_b1 = self.in_b1.with_exported(|| {
                    self.in_b1.set_value(1)?;
                    Ok(())
                });
                let res_b2 = self.in_b2.with_exported(|| {
                    self.in_b2.set_value(0)?;
                    Ok(())
                });
                res_b1.and(res_b2)
            }
        }
    }

    fn set_reverse(&self, device: Device) -> gpio::Result<()> {
        match device {
            Device::A => {
                let res_a1 = self.in_a1.with_exported(|| {
                    self.in_a1.set_value(0)?;
                    Ok(())
                });
                let res_a2 = self.in_a2.with_exported(|| {
                    self.in_a2.set_value(1)?;
                    Ok(())
                });
                res_a1.and(res_a2)
            },
            Device::B => {
                let res_b1 = self.in_b1.with_exported(|| {
                    self.in_b1.set_value(0)?;
                    Ok(())
                });
                let res_b2 = self.in_b2.with_exported(|| {
                    self.in_b2.set_value(1)?;
                    Ok(())
                });
                res_b1.and(res_b2)
            }
        }
    }
}
