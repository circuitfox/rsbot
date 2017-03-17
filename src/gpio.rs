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
