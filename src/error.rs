use std::error;
use std::fmt;

use sysfs_gpio as gpio;

#[derive(Debug)]
pub enum Error {
    Gpio(gpio::Error)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Gpio(ref err) => err.fmt(f)
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Gpio(ref err) => err.description()
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Gpio(ref err) => Some(err)
        }
    }
}

impl From<gpio::Error> for Error {
    fn from(err: gpio::Error) -> Error {
        Error::Gpio(err)
    }
}
