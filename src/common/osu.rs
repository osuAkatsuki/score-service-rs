pub fn format_bool(value: bool) -> &'static str {
    match value {
        true => "True",
        false => "False",
    }
}

pub const DATETIME_FORMAT_STR: &str = "%y%m%d%H%M%S";
