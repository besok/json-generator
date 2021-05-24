//! ### Generators
//! The functions which are responsible to generate new json values after parsing.
pub mod generators;
pub mod from_string;

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
use crate::generator::generators::RandomArray;
use crate::error::GenError;

/// The trait represents the function to generate jsons
pub trait GeneratorFunc {
    /// the method generates a new json value
    fn next_value(&mut self) -> Value;
    /// the method carries a logic how to merge two functions into one.
    /// It can be useful for the compound functions like `RandomArray`
    fn merge(&self, another_gf: Func) -> Result<Func, GenError> {
        Err(GenError::new_with("the functions are unable to merge in the order"))
    }
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

pub fn new_func<T: GeneratorFunc + 'static>(entity: T) -> Func {
    Rc::new(RefCell::new(entity))
}

/// In general, that is a wrapper on the function `GeneratorFunc`
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
        debug!("create a generator({})", print_type_of(&entity));
        Generator { function: new_func(entity) }
    }
    pub fn next(&self) -> Value {
        RefCell::borrow_mut(&self.function).next_value()
    }


    pub fn merge(&self, gen: &Generator) -> Result<Generator, GenError> {
        RefCell::borrow_mut(&self.function)
            .merge(gen.function.clone())
            .map(|e| Generator { function: e })
    }
}


#[cfg(test)]
mod tests {
    use crate::generator::{GeneratorFunc, Generator};
    use serde_json::Value;

    struct SimpleGenFun {}

    impl GeneratorFunc for SimpleGenFun {
        fn next_value(&mut self) -> Value {
            Value::Null
        }
    }

    #[test]
    fn to_string_test() {
        let f = Generator::new(SimpleGenFun {});
        if_let!(f.next() => f.next() => assert_eq!(f.next(),Value::Null))
    }
}