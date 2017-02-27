use std::error;
use std::fmt;

use sysfs_gpio as gpio;

#[derive(Debug)]
pub enum Error {
    Gpio(gpio::Error),
    Build(BuilderError),
}

#[derive(Debug)]
pub enum BuilderError {
    FrontMotorPins,
    RearMotorPins,
    FrontDistancePins,
    RearDistancePins,
    LeftDistancePins,
    RightDistancePins,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Gpio(ref err) => err.fmt(f),
            Error::Build(ref err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Gpio(ref err) => err.description(),
            Error::Build(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Gpio(ref err) => Some(err),
            Error::Build(ref err) => Some(err),
        }
    }
}

impl From<gpio::Error> for Error {
    fn from(err: gpio::Error) -> Error {
        Error::Gpio(err)
    }
}

impl From<BuilderError> for Error {
    fn from(err: BuilderError) -> Error {
        Error::Build(err)
    }
}

impl error::Error for BuilderError {
    fn description(&self) -> &str {
        "Error building controller"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl fmt::Display for BuilderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Builder: {} not specified", match *self {
            BuilderError::FrontMotorPins => "front motor controller pins",
            BuilderError::RearMotorPins => "rear motor controller pins",
            BuilderError::FrontDistancePins => "front distance sensor pins",
            BuilderError::RearDistancePins => "rear distance sensor pins",
            BuilderError::LeftDistancePins => "left distance sensor pins",
            BuilderError::RightDistancePins => "right distance sensor pins",
        })
    }
}
