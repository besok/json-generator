#[macro_export]
macro_rules! if_let {
    ( $gen_res:expr => $exp_res:pat => $next_check:expr) => {
        if let $exp_res = $gen_res {
            $next_check
        } else{
            panic!(format!("the epr {:?} is not equal to expected one",$gen_res))
        }

    };

    ($left:expr => $right:expr => $next_check:expr) => {
        if $left == $right {
          $next_check
        } else{
            panic!(format!("the left {:?} is not equal to the right",$left))
        }

    }

}

#[cfg(test)]
mod tests {
    use crate::generator::generators::{RandomString, RandomInt};
    use serde_json::Value;
}