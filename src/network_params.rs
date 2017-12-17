use port_parser::*;

use std::net::{lookup_host, LookupHost};
use std::fmt;
use std::error;
use std::error::Error;
use std::io;

pub fn get_network_params(l_port: &str, r_host: &str, r_port: &str)
                      -> Result<(u16, LookupHost, u16), NetworkParamsError> {
    let (l_port, r_port) = parse_ports(l_port, r_port).map_err(|e| {
        NetworkParamsError::Parse(e)
    })?;

    let r_host = lookup_host(&r_host).map_err(|e| {
        NetworkParamsError::Lookup(e.into())
    })?;

    Ok((l_port, r_host, r_port))
}

pub fn parse_ports(l_port: &str, r_port: &str) -> Result<(u16, u16), PortParseError> {
    let l_port = parse_port(l_port)?;
    let r_port = parse_port(r_port)?;

    Ok((l_port, r_port))
}

#[derive(Debug)]
pub enum NetworkParamsError {
    Parse(PortParseError),
    Lookup(io::Error),
}

impl fmt::Display for NetworkParamsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bad network parameters: {:?}", self.cause())
    }
}

impl error::Error for NetworkParamsError {
    fn description(&self) -> &str {
        "bad network params"
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &NetworkParamsError::Parse(ref e) => Some(e),
            &NetworkParamsError::Lookup(ref e) => Some(e),
        }
    }
}
