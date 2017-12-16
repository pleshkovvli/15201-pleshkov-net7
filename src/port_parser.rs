use std::fmt;
use std::error;

pub fn parse_port(port: &str) -> Result<u16, PortParseError> {
    Ok(port.parse::<u16>().map_err(|_| {
        PortParseError::new(port)
    })?)
}

#[derive(Debug, Clone)]
pub struct PortParseError {
    given_string: String
}

impl PortParseError {
    fn new(given_string: &str) -> PortParseError {
        PortParseError {
            given_string: String::from(given_string)
        }
    }
}

impl fmt::Display for PortParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} is not a port number", self.given_string)
    }
}

impl error::Error for PortParseError {
    fn description(&self) -> &str {
        "invalid port value"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
