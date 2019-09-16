use std::collections::HashMap;

use rocket_contrib::templates::tera::{self, Value, Number};

/// Convert seconds into a hour/minute duration string.
pub fn duration(value: Value, _: HashMap<String, Value>) -> tera::Result<Value> {
    let num: Number = match value {
        Value::Number(num) => num,
        _ => return Err("The duration filter can only be applied to numbers".into()),
    };
    let seconds: u64 = match num.as_u64() {
        Some(secs) => secs,
        None => return Err("The duration filter can only be applied to integers".into()),
    };
    let hours = seconds / 3600;
    let minutes = seconds / 60 % 60;
    Ok(Value::String(format!("{:02}:{:02}", hours, minutes)))
}
