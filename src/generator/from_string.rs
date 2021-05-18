pub trait FromStringTo: Sized {
    fn parse(v: &str, trim_spaces: bool) -> Result<Self, String>;
}

impl FromStringTo for String {
    fn parse(v: &str, rem_spaces: bool) -> Result<Self, String> {
        Ok(
            if rem_spaces { trim_spaces(v) } else { String::from(v) }
        )
    }
}

impl FromStringTo for i64 {
    fn parse(v: &str, rem_spaces: bool) -> Result<Self, String> {
        let value = if rem_spaces { trim_spaces(v) } else { v.to_string() };
        value.parse::<i64>().map_err(|e| e.to_string())
    }
}

impl FromStringTo for i32 {
    fn parse(v: &str, rem_spaces: bool) -> Result<Self, String> {
        let value = if rem_spaces { trim_spaces(v) } else { v.to_string() };
        value.parse::<i32>().map_err(|e| e.to_string())
    }
}impl FromStringTo for usize {
    fn parse(v: &str, rem_spaces: bool) -> Result<Self, String> {
        let value = if rem_spaces { trim_spaces(v) } else { v.to_string() };
        value.parse::<usize>().map_err(|e| e.to_string())
    }
}

fn trim_spaces(v: &str) -> String {
    v.replace(" ", "")
}