use std::thread;
use std::time;

use sysfs_gpio as gpio;
use sysfs_gpio::Pin;

const SOUND_SPEED_CM: u64 = 34300;

pub struct Sensor {
    trigger: Pin,
    echo: Pin
}

impl Drop for Sensor {
    fn drop(&mut self) {
        gpio_unexport!(self, { trigger, echo });
    }
}

impl Sensor {
    pub fn new(trigger: u64, echo: u64) -> gpio::Result<Sensor> {
        let sensor = Sensor {
            trigger: Pin::new(trigger),
            echo: Pin::new(echo)
        };
        gpio_export!(sensor, { trigger, echo });
        sensor.trigger.set_direction(gpio::Direction::Out)?;
        sensor.echo.set_direction(gpio::Direction::In)?;
        Ok(sensor)
    }
    
    pub fn value(&self) -> gpio::Result<f32> {
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
        let travel_time = (travel_dur.as_secs() as f32 +
                           (travel_dur.subsec_nanos() as f32) / 1000000000f32) / 2f32;
        Ok(travel_time * SOUND_SPEED_CM as f32)
    }
}
