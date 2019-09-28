use std::collections::HashMap;

use rocket_contrib::templates::tera::{self, Number, Value};

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

/// Convert an xcontext tracktype into an icon.
///
/// If the string value is not a valid tracktype, the value is returned
/// unmodified.
pub fn xcontest_icon(value: Value, _: HashMap<String, Value>) -> tera::Result<Value> {
    let tracktype: String = if let Value::String(s) = value {
        s
    } else {
        return Err("The xcontest_icon filter can only be applied to strings".into());
    };
    Ok(Value::String(match &*tracktype {
        "fai_triangle" | "flat_triangle" | "free_flight" => format!(
            "<img src=\"/static/img/xcontest_{0}.gif\" alt=\"{0}\" title=\"{0}\" class=\"xcontest-tracktype\">",
            tracktype
        ),
        _ => tracktype,
    }))
}
