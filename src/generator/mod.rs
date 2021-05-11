//! ### Generators
//! The functions which are responsible to generate new json values
pub mod generators;

use std::fmt::{Debug, Formatter, Error};
use std::collections::HashMap;
use std::thread::Thread;
use rand::prelude::ThreadRng;
use std::ops::Range;
use rand::Rng;
use rand::distributions::Alphanumeric;
use uuid::Uuid;
use std::cell::RefCell;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::rc::Rc;
use std::any::{type_name, Any};
use serde_json::Value;

/// The trait represents the function to generate jsons
pub trait GeneratorFunc {
    fn next_value(&mut self) -> Value;
}

/// for logging purposes
pub fn print_type_of<T>(_: &T) -> String {
    format!("{}", std::any::type_name::<T>())
}

impl ToString for dyn GeneratorFunc {
    fn to_string(&self) -> String {
        format!("GeneratorFunc[{:?}]", print_type_of(&self))
    }
}

impl Debug for dyn GeneratorFunc {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(self.to_string().as_str());
        Ok(())
    }
}

type Func = Rc<RefCell<dyn GeneratorFunc>>;

/// the struct represents the generator which is essentially a wrapper to generalize GeneratorFunc
#[derive(Debug)]
pub struct Generator {
    function: Func
}

impl ToString for Generator {
    fn to_string(&self) -> String {
        format!("Generator[{:?}]", self.function.clone().borrow().to_string())
    }
}


impl Clone for Generator {
    fn clone(&self) -> Self {
        Generator { function: self.function.clone() }
    }
}

impl Generator {
    pub fn new<T: GeneratorFunc + 'static>(entity: T) -> Self {
        info!("create a generator({})", print_type_of(&entity));
        Generator { function: Rc::new(RefCell::new(entity)) }
    }
    pub fn next(&self) -> Value {
        RefCell::borrow_mut(&self.function).next_value()
    }
}


#[cfg(test)]
mod tests {}