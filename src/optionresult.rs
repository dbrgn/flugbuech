use std::borrow::Cow;

use chrono::naive::{NaiveDate, NaiveTime};
use rocket::form::{self, FromFormField, ValueField};

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

/// Parse naive dates in ISO format (YYYY-MM-DD).
///
/// Return None if the field is empty, Some if the field can be parsed, and Err
/// if the field cannot be parsed.
#[rocket::async_trait]
impl<'r> FromFormField<'r> for OptionResult<NaiveDate> {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        if field.value.trim().is_empty() {
            return Ok(OptionResult::None);
        }
        NaiveDate::parse_from_str(field.value, "%Y-%m-%d")
            .map(OptionResult::Ok)
            .or_else(|e| {
                Ok(OptionResult::Err(format!(
                    "Invalid date in field {} ({}): {}",
                    field.name, field.value, e
                )))
            })
    }
}

/// Parse urlencoded naive times in format HH:MM:SS or HH:MM.
///
/// Return None if the field is empty, Some if the field can be parsed, and Err
/// if the field cannot be parsed.
impl<'r> FromFormField<'r> for OptionResult<NaiveTime> {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        if field.value.trim().is_empty() {
            return Ok(OptionResult::None);
        }
        let value = if field.value.len() < 8 {
            Cow::Owned(format!("{}:00", field.value))
        } else {
            Cow::Borrowed(field.value)
        };
        NaiveTime::parse_from_str(&value, "%H:%M:%S")
            .map(OptionResult::Ok)
            .or_else(|e| {
                Ok(OptionResult::Err(format!(
                    "Invalid time in field {} ({}): {}",
                    field.name, value, e
                )))
            })
    }
}

/// Parse integers.
///
/// Return None if the field is empty, Some if the field can be parsed, and Err
/// if the field cannot be parsed.
impl<'r> FromFormField<'r> for OptionResult<i32> {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        if field.value.trim().is_empty() {
            return Ok(OptionResult::None);
        }
        field.value.parse().map(OptionResult::Ok).or_else(|e| {
            Ok(OptionResult::Err(format!(
                "Invalid integer in field {}: {}",
                field.name, e
            )))
        })
    }
}

/// Parse floating point numbers.
///
/// Return None if the field is empty, Some if the field can be parsed, and Err
/// if the field cannot be parsed.
impl<'r> FromFormField<'r> for OptionResult<f32> {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        if field.value.trim().is_empty() {
            return Ok(OptionResult::None);
        }
        field.value.parse().map(OptionResult::Ok).or_else(|e| {
            Ok(OptionResult::Err(format!(
                "Invalid float in field {}: {}",
                field.name, e
            )))
        })
    }
}

/// Parse base64 encoded data.
///
/// Return None if the field is empty, Some if the field can be parsed, and Err
/// if the field cannot be parsed.
impl<'r> FromFormField<'r> for OptionResult<Base64Data> {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        if field.value.trim().is_empty() {
            return Ok(OptionResult::None);
        }
        base64::decode_config(field.value, base64::URL_SAFE)
            .map(|vec| OptionResult::Ok(Base64Data(vec)))
            .or_else(|e| {
                Ok(OptionResult::Err(format!(
                    "Invalid base64 data in field {}: {}",
                    field.name, e
                )))
            })
    }
}
