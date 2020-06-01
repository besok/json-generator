#[macro_export]
macro_rules! if_let {
    ( $gen_res:expr => $exp_res:pat => $next_check:expr) => {
        if let $exp_res = $gen_res {
            $next_check
        } else{
            panic!("the initial epr is not equal to expected one")
        }

    }
}


#[cfg(test)]
mod tests {
    use crate::generator::generators::{RandomString, RandomInt};
    use crate::parser::Json;
    use crate::generator::GeneratorFunc;


    #[test]
    fn simple_test() {
        if_let!(RandomInt::new(10,20).next() => Json::Num(el) => assert_eq!(el > 9, el < 20));
    }
}