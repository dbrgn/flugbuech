use chrono::naive::{NaiveDate, NaiveTime};
use rocket::{http::RawStr, request::FromFormValue};

use crate::base64::Base64Data;

/// A combined Option / Result type.
#[derive(Debug)]
pub enum OptionResult<T> {
    None,
    Ok(T),
    Err(String),
}

impl<T> OptionResult<T> {
    pub fn into_result(self) -> Result<Option<T>, String> {
        match self {
            Self::None => Ok(None),
            Self::Ok(v) => Ok(Some(v)),
            Self::Err(e) => Err(e),
        }
    }
}

impl<'v> FromFormValue<'v> for OptionResult<NaiveDate> {
    type Error = String;
    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        if form_value.trim().is_empty() {
            return Ok(OptionResult::None);
        }
        NaiveDate::parse_from_str(form_value, "%Y-%m-%d")
            .map(OptionResult::Ok)
            .or_else(|e| Ok(OptionResult::Err(format!("Invalid date ({}): {}", form_value, e))))
    }
}

impl<'v> FromFormValue<'v> for OptionResult<NaiveTime> {
    type Error = !;
    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        if form_value.trim().is_empty() {
            return Ok(OptionResult::None);
        }
        let mut decoded = match form_value.url_decode() {
            Ok(val) => val,
            Err(e) => return Ok(OptionResult::Err(format!("Could not urldecode value: {}", e))),
        };
        if decoded.len() < 8 {
            decoded.push_str(":00");
        }
        NaiveTime::parse_from_str(&decoded, "%H:%M:%S")
            .map(OptionResult::Ok)
            .or_else(|e| Ok(OptionResult::Err(format!("Invalid time ({}): {}", decoded, e))))
    }
}

impl<'v> FromFormValue<'v> for OptionResult<i32> {
    type Error = !;
    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        if form_value.trim().is_empty() {
            return Ok(OptionResult::None);
        }
        form_value
            .parse()
            .map(OptionResult::Ok)
            .or_else(|e| Ok(OptionResult::Err(format!("Invalid integer: {}", e))))
    }
}

impl<'v> FromFormValue<'v> for OptionResult<f32> {
    type Error = !;
    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        if form_value.trim().is_empty() {
            return Ok(OptionResult::None);
        }
        form_value
            .parse()
            .map(OptionResult::Ok)
            .or_else(|e| Ok(OptionResult::Err(format!("Invalid integer: {}", e))))
    }
}

impl<'v> FromFormValue<'v> for OptionResult<Base64Data> {
    type Error = !;
    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        if form_value.trim().is_empty() {
            return Ok(OptionResult::None);
        }
        base64::decode_config(form_value, base64::URL_SAFE)
            .map(|vec| OptionResult::Ok(Base64Data(vec)))
            .or_else(|e| Ok(OptionResult::Err(format!("Invalid base64 data: {}", e))))
    }
}
