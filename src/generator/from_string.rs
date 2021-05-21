use crate::error::GenError;

pub trait FromStringTo: Sized {
    fn parse(v: &str, trim_spaces: bool) -> Result<Self, GenError>;
}

impl FromStringTo for String {
    fn parse(v: &str, rem_spaces: bool) -> Result<Self, GenError> {
        Ok(
            if rem_spaces { trim_spaces(v) } else { String::from(v) }
        )
    }
}

impl FromStringTo for i64 {
    fn parse(v: &str, rem_spaces: bool) -> Result<Self, GenError> {
        let value = if rem_spaces { trim_spaces(v) } else { v.to_string() };
        value.parse::<i64>()
            .map_err(|e| GenError::new_with_in_parser(
                format!("impossible to convert string to i64 due to {}", e.to_string()).as_str()
            ))
    }
}

impl FromStringTo for i32 {
    fn parse(v: &str, rem_spaces: bool) -> Result<Self, GenError> {
        let value = if rem_spaces { trim_spaces(v) } else { v.to_string() };
        value.parse::<i32>()
            .map_err(|e| GenError::new_with_in_parser(
                format!("impossible to convert string to i32 due to {}", e.to_string()).as_str()
            ))
    }
}

impl FromStringTo for usize {
    fn parse(v: &str, rem_spaces: bool) -> Result<Self, GenError> {
        let value = if rem_spaces { trim_spaces(v) } else { v.to_string() };
        value.parse::<usize>()
            .map_err(|e| GenError::new_with_in_parser(
                format!("impossible to convert string to usize due to {}", e.to_string()).as_str()
            ))
    }
}

fn trim_spaces(v: &str) -> String {
    v.replace(" ", "")
}