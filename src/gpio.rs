// The 200ms delay is required to allow permissions to be changed on the exported pins.
macro_rules! gpio_export {
    ($s: ident, {$($gpio:ident),+}) => ({
        use std::thread;
        use std::time;
        $(
            $s.$gpio.export()?;
        )+
        thread::sleep(time::Duration::from_millis(200));
    });
}

// TODO: Log any Errors
macro_rules! gpio_unexport {
    ($s: ident, {$($gpio:ident),+}) => ({
            $(
                $s.$gpio.unexport().ok();
             )+
    });
}
