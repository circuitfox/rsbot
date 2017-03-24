use std::thread;
use std::time;

use sysfs_gpio as gpio;
use sysfs_gpio::Pin;

use super::Result;

const SOUND_SPEED_CM: u64 = 34300;

#[derive(Clone, Debug)]
pub struct Sensor {
    trigger: Pin,
    echo: Pin,
}

impl Sensor {
    pub fn new(trigger: Pin, echo: Pin) -> Result<Sensor> {
        let sensor = Sensor {
            trigger: trigger,
            echo: echo,
        };
        sensor.trigger.set_direction(gpio::Direction::Out)?;
        sensor.echo.set_direction(gpio::Direction::In)?;
        Ok(sensor)
    }

    pub fn value(&self) -> Result<f32> {
        // 10μs pulse
        self.trigger.set_value(1)?;
        thread::sleep(time::Duration::new(0, 10000));
        self.trigger.set_value(0)?;

        let mut pulse_start = time::Instant::now();
        let mut pulse_end = time::Instant::now();
        while self.echo.get_value()? == 0 {
            pulse_start = time::Instant::now();
        }
        while self.echo.get_value()? == 1 {
            pulse_end = time::Instant::now();
        }
        let travel_dur = pulse_end.duration_since(pulse_start);
        let travel_time = travel_dur.as_secs() as f32 +
                          travel_dur.subsec_nanos() as f32 / 1e9f32 / 2f32;
        Ok(travel_time * SOUND_SPEED_CM as f32)
    }

    pub fn unexport(&mut self) {
        self.trigger.set_value(0).ok();
        self.echo.set_value(0).ok();
        gpio_unexport!(self, {trigger, echo})
    }
}
