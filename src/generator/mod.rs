pub mod generators;

use std::fmt::{Debug, Formatter, Error};
use crate::parser::Json;
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

pub trait GeneratorFunc {
    fn next(&mut self) -> Json;
}

impl ToString for dyn GeneratorFunc{
    fn to_string(&self) -> String {
        format!("GeneratorFunc[{:?}]",self.type_id())
    }
}
impl Debug for dyn GeneratorFunc {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(self.to_string().as_str());
        Ok(())
    }
}

#[derive(Debug)]
pub struct Generator {
    delegate: Rc<RefCell<dyn GeneratorFunc>>
}

impl ToString for Generator{
    fn to_string(&self) -> String {
        format!("Generator[{:?}]",self.delegate.clone().borrow().to_string())
    }
}


impl Clone for Generator{
    fn clone(&self) -> Self {
        Generator{ delegate: self.delegate.clone() }
    }
}

impl Generator  {
    pub fn new<T:GeneratorFunc + 'static>(entity: T) -> Self {
        Generator { delegate: Rc::new(RefCell::new(entity)) }
    }
    pub fn next(&self) -> Json {
        RefCell::borrow_mut(&self.delegate).next()
    }
}




#[cfg(test)]
mod tests {
    use crate::parser::Json;
    use crate::generator::generators::Constant;


}