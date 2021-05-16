use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use nom::error::ErrorKind;

use crate::generator::Generator;
use crate::parser::generator::GenError;

pub mod json;
pub mod generator;
