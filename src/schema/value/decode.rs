use crate::schema::dialect::SQLDialect;
use teo_runtime::value::Value;
use chrono::{NaiveDate, DateTime, Utc};
use indexmap::IndexMap;
use quaint_forked::prelude::{ResultRow, ResultSet, Value as QuaintValue};
use teo_parser::r#type::Type;

pub(crate) struct RowDecoder { }

impl RowDecoder {

    pub(crate) fn decode_raw(value: &quaint_forked::Value) -> Value {
        match value {
            quaint_forked::Value::Float(f) => {
                match f {
                    Some(f) => Value::Float32(*f),
                    None => Value::Null,
                }
            }
            quaint_forked::Value::Double(d) => {
                match d {
                    Some(d) => Value::Float(*d),
                    None => Value::Null,
                }
            }
            quaint_forked::Value::Text(t) => {
                match t {
                    Some(d) => Value::String(d.as_ref().to_owned()),
                    None => Value::Null,
                }
            }
            quaint_forked::Value::Boolean(b) => {
                match b {
                    Some(d) => Value::Bool(*d),
                    None => Value::Null,
                }
            }
            quaint_forked::Value::Date(d) => {
                match d {
                    Some(d) => Value::Date(d.clone()),
                    None => Value::Null,
                }
            }
            quaint_forked::Value::DateTime(d) => {
                match d {
                    Some(d) => Value::DateTime(d.clone()),
                    None => Value::Null,
                }
            }
            quaint_forked::Value::Int32(i) => {
                match i {
                    Some(i) => Value::Int(*i),
                    None => Value::Null,
                }
            }
            quaint_forked::Value::Int64(i) => {
                match i {
                    Some(i) => Value::Int64(*i),
                    None => Value::Null,
                }
            }
            quaint_forked::Value::Numeric(d) => {
                match d {
                    Some(i) => Value::Decimal(i.clone()),
                    None => Value::Null,
                }
            }
            _ => unreachable!()
        }
    }

    pub(crate) fn decode_raw_result_set(set: ResultSet) -> Value {
        let columns = set.columns().clone();
        let results: Vec<Value> = set.into_iter().map(|row| {
            let mut map: IndexMap<String, Value> = IndexMap::new();
            for column in columns.iter() {
                let val = row.get(column).unwrap();
                map.insert(column.to_owned(), Self::decode_raw(val));
            }
            Value::Dictionary(map)
        }).collect();
        Value::Array(results)
    }

    pub(crate) fn decode_serial(optional: bool, row: &ResultRow, column_name: &str) -> Value {
        let try_value = row.get(column_name);
        if try_value.is_none() && optional {
            Value::Null
        } else {
            if try_value.is_none() {
                Value::Null
            } else {
                let val = try_value.unwrap();
                if val.is_i32() {
                    Value::Int(val.as_i32().unwrap())
                } else {
                    Value::Int64(val.as_i64().unwrap())
                }
            }
        }
    }

    pub(crate) fn decode_value(r#type: &Type, optional: bool, value: Option<&quaint_forked::Value>, dialect: SQLDialect) -> Value {
        if optional {
            if value.is_none() {
                return Value::Null;
            }
        }
        let r#type = r#type.unwrap_optional();
        let value = value.unwrap();
        if r#type.is_bool() {
            if let Some(v) = value.as_bool() {
                return Value::Bool(v)
            } else {
                return Value::Null;
            }
        }
        if r#type.is_string() {
            if let Some(v) = value.as_str() {
                return Value::String(v.to_owned())
            } else {
                return Value::Null;
            }
        }
        if r#type.is_int() {
            if let Some(v) = value.as_i32() {
                return Value::Int(v);
            } else {
                return Value::Null;
            }
        }
        if r#type.is_int64() {
            if let Some(v) = value.as_i64() {
                return Value::Int64(v);
            } else if let Some(v) = value.as_i32() {
                return Value::Int(v);
            } else {
                return Value::Null;
            }
        }
        if r#type.is_float32() || r#type.is_float() {
            if let Some(f64_val) = value.as_f64() {
                return if r#type.is_float() {
                    Value::Float(f64_val)
                } else if r#type.is_float32() {
                    Value::Float32(f64_val as f32)
                } else {
                    unreachable!()
                };
            } else if let Some(f32_val) = value.as_f32() {
                return if r#type.is_float() {
                    Value::Float(f32_val as f64)
                } else if r#type.is_float32() {
                    Value::Float32(f32_val)
                } else {
                    unreachable!()
                };
            } else {
                return Value::Null;
            }
        }
        if r#type.is_date() {
            if dialect == SQLDialect::PostgreSQL {
                if let Some(naive_date) = value.as_date() {
                    return Value::Date(naive_date);
                } else {
                    return Value::Null;
                }
            } else if dialect == SQLDialect::SQLite {
                if let Some(timestamp) = value.as_str() {
                    let naive_date = NaiveDate::parse_from_str(timestamp, "%Y-%m-%d").unwrap();
                    return Value::Date(naive_date);
                } else {
                    return Value::Null;
                }
            } else if dialect == SQLDialect::MySQL {
                if let Some(datetime) = value.as_datetime() {
                    let naive_date = datetime.date_naive();
                    return Value::Date(naive_date);
                } else {
                    return Value::Null;
                }
            } else {
                if let Some(naive_date) = value.as_date() {
                    return Value::Date(naive_date);
                } else {
                    return Value::Null;
                }
            }
        }
        if r#type.is_datetime() {
            if dialect == SQLDialect::PostgreSQL {
                if let Some(datetime) = value.as_datetime() {
                    return Value::DateTime(datetime);
                } else {
                    return Value::Null;
                }
            } else if dialect == SQLDialect::SQLite {
                if let Some(timestamp) = value.as_str() {
                    return Value::DateTime(DateTime::parse_from_rfc3339(timestamp).unwrap().with_timezone(&Utc));
                } else {
                    return Value::Null;
                }
            } else {
                if let Some(datetime) = value.as_datetime() {
                    return Value::DateTime(datetime);
                } else {
                    return Value::Null;
                }
            }
        }
        if r#type.is_decimal() {
            if let Some(val) = value.as_numeric() {
                return Value::Decimal(val.clone());
            } else {
                return Value::Null;
            }
        }
        if r#type.is_array() {
            if let Some(vals) = value.as_array() {
                let inner = r#type.as_array().unwrap();
                return Value::Array(vals.iter().map(|v| Self::decode_value(inner.unwrap_optional(), inner.is_optional(), Some(v), dialect)).collect());
            } else {
                return Value::Null;
            }
        }
        if r#type.is_enum_variant() {
            match value {
                QuaintValue::Enum(v) => {
                    if let Some(v) = v {
                        return Value::String(v.as_ref().to_owned());
                    } else {
                        return Value::Null;
                    }
                }
                QuaintValue::Text(v) => {
                    if let Some(v) = v {
                        return Value::String(v.as_ref().to_owned());
                    } else {
                        return Value::Null;
                    }
                }
                _ => panic!("unhandled enum variant"),
            }
        }
        panic!("Unhandled database when decoding type.")
    }

    pub(crate) fn decode(r#type: &Type, optional: bool, row: &ResultRow, column_name: &str, dialect: SQLDialect) -> Value {
        let result = row.get(column_name);
        Self::decode_value(r#type, optional, result.clone(), dialect)
    }
}
