pub mod generators;

use std::fmt::Debug;
use crate::parser::Json;
use std::collections::HashMap;
use std::thread::Thread;
use rand::prelude::ThreadRng;
use std::ops::Range;
use rand::Rng;
use rand::distributions::Alphanumeric;
use uuid::Uuid;

trait Generator {
    fn next(&mut self) -> Json;
}

pub struct Generators {
    idx: usize,
    delegate: HashMap<usize, Box<dyn Generator>>,
}

impl Generators {
    pub fn new() -> Self {
        Generators { idx: 0, delegate: HashMap::new() }
    }

    pub fn add_generator(&mut self, g: Box<dyn Generator>) -> Result<usize, String> {
        self.idx += 1;
        self.delegate.insert(self.idx, g);
        Ok(self.idx)
    }

    pub fn generate(&mut self, idx: usize) -> Result<Json, String> {
        match self.delegate.get_mut(&idx) {
            None => Err("the key not found".to_string()),
            Some(g) => Ok(g.next()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::Json;
    use crate::generator::Generators;
    use crate::generator::generators::Constant;

    #[test]
    fn test() {
        let mut generators = Generators::new();
        if let Ok(i1) = generators.add_generator(Box::from(Constant { value: "test".to_string() })) {
            if let Ok(i2) = generators.add_generator(Box::from(Constant { value: 1 as i64 })) {
                assert_eq!(generators.generate(i1), Ok(Json::Str("test".to_string())));
                assert_eq!(generators.generate(i2), Ok(Json::Num(1)));
            } else {
                assert_eq!(true, false)
            }
        } else {
            assert_eq!(true, false)
        }
    }
}