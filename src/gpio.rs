use retry;
use sysfs_gpio as gpio;

use error;
use super::Result;

/// Exports one or more GPIO pins wrapped in an `Option`.
///
/// The specified pins are assumed to be fields of `$p`.
///
/// After calling this function, the newly exported pins will be owned by root:root
/// for an unspecified amount of time, before ownership changes to root:gpio,
/// so `poll_pin_init` should be called afterward to allow ownership to change.
macro_rules! gpio_export {
    ($p: ident, {$($opt_gpio: ident),+}) => ({
        $(
            if let Some(ref gpio) = $p.$opt_gpio {
                gpio.export()?;
            }
        )+
    });
}

/// Unexports one or more GPIO pins.
///
/// The specified pins are assumed to be fields of `$p`.
// TODO: Log any Errors
macro_rules! gpio_unexport {
    ($p: ident, {$($gpio: ident),+}) => ({
        $(
            $p.$gpio.unexport().ok();
        )+
    });
}

/// Sets one or more GPIO pins as outputs.
///
/// The specified pins are assumed to be fields of `$p`.
macro_rules! gpio_out {
    ($p: ident, {$($gpio: ident),+}) => ({
        $(
            $p.$gpio.set_direction(gpio::Direction::Out)?;
        )+
    });
}

/// Poll the given pin to see if we can access it. This function polls for the value
/// of the pin every 50ms for 500ms before giving up.
pub fn poll_pin_init(pin: &gpio::Pin) -> Result<()> {
    // We need to convert from RetryError to our Error, and we don't care about the
    // result of get_value()
    retry::retry(10, 50, || {
        pin.get_value()
    }, |res| res.is_ok())
    .map_err(|_| error::Error::Build(error::BuilderError::ExportError))
    .and_then(|_| Ok(()))
}
