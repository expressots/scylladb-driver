use std::net::IpAddr;

use scylla::value::CqlValue;
use serde_json::{Number, Value};
use uuid::Uuid;

use crate::error::driver_error;

pub fn cql_value_to_json(value: &CqlValue) -> Value {
  match value {
    CqlValue::Ascii(s) | CqlValue::Text(s) => Value::String(s.clone()),
    CqlValue::Boolean(b) => Value::Bool(*b),
    CqlValue::Int(i) => Value::Number(Number::from(*i)),
    CqlValue::BigInt(i) => Value::Number(Number::from(*i)),
    CqlValue::SmallInt(i) => Value::Number(Number::from(*i)),
    CqlValue::TinyInt(i) => Value::Number(Number::from(*i)),
    CqlValue::Float(f) => json_number_from_f64(*f as f64),
    CqlValue::Double(d) => json_number_from_f64(*d),
    CqlValue::Uuid(uuid) => Value::String(uuid.to_string()),
    CqlValue::Timeuuid(timeuuid) => Value::String(timeuuid.to_string()),
    CqlValue::Blob(bytes) => Value::Object(serde_json::Map::from_iter([
      ("type".into(), Value::String("Buffer".into())),
      (
        "data".into(),
        Value::Array(
          bytes
            .iter()
            .map(|b| Value::Number(Number::from(*b)))
            .collect(),
        ),
      ),
    ])),
    CqlValue::Timestamp(ts) => Value::Number(Number::from(ts.0)),
    CqlValue::Date(date) => Value::Number(Number::from(date.0)),
    CqlValue::Time(time) => Value::Number(Number::from(time.0)),
    CqlValue::Inet(addr) => Value::String(addr.to_string()),
    CqlValue::Counter(counter) => Value::Number(Number::from(counter.0)),
    CqlValue::Decimal(decimal) => {
      let (bytes, scale) = decimal.as_signed_be_bytes_slice_and_exponent();
      let int_val = bytes_to_bigint_string(bytes);
      let scale = scale as i64;
      if scale <= 0 {
        let zeros = "0".repeat((-scale) as usize);
        Value::String(format!("{}{}", int_val, zeros))
      } else {
        let is_negative = int_val.starts_with('-');
        let digits = if is_negative { &int_val[1..] } else { &int_val };
        let result = if (scale as usize) >= digits.len() {
          let pad = "0".repeat(scale as usize - digits.len());
          format!("0.{}{}", pad, digits)
        } else {
          let pos = digits.len() - scale as usize;
          format!("{}.{}", &digits[..pos], &digits[pos..])
        };
        Value::String(if is_negative {
          format!("-{}", result)
        } else {
          result
        })
      }
    }
    CqlValue::Varint(varint) => {
      Value::String(bytes_to_bigint_string(varint.as_signed_bytes_be_slice()))
    }
    CqlValue::Duration(duration) => Value::Object(serde_json::Map::from_iter([
      ("months".into(), Value::Number(Number::from(duration.months))),
      ("days".into(), Value::Number(Number::from(duration.days))),
      (
        "nanoseconds".into(),
        Value::Number(Number::from(duration.nanoseconds)),
      ),
    ])),
    CqlValue::List(values) | CqlValue::Set(values) | CqlValue::Vector(values) => {
      Value::Array(values.iter().map(cql_value_to_json).collect())
    }
    CqlValue::Map(entries) => {
      let mut map = serde_json::Map::new();
      for (key, value) in entries {
        map.insert(json_key_string(key), cql_value_to_json(value));
      }
      Value::Object(map)
    }
    CqlValue::Tuple(values) => Value::Array(
      values
        .iter()
        .map(|value| match value {
          Some(value) => cql_value_to_json(value),
          None => Value::Null,
        })
        .collect(),
    ),
    CqlValue::UserDefinedType { name, fields, .. } => {
      let mut map = serde_json::Map::new();
      map.insert("__udt_name".into(), Value::String(name.clone()));
      for (field_name, field_value) in fields {
        map.insert(
          field_name.clone(),
          match field_value {
            Some(value) => cql_value_to_json(value),
            None => Value::Null,
          },
        );
      }
      Value::Object(map)
    }
    CqlValue::Empty => Value::Null,
    _ => Value::String(format!("{:?}", value)),
  }
}

fn json_key_string(value: &CqlValue) -> String {
  match value {
    CqlValue::Ascii(s) | CqlValue::Text(s) => s.clone(),
    other => cql_value_to_json(other).to_string(),
  }
}

fn json_number_from_f64(value: f64) -> Value {
  Number::from_f64(value)
    .map(Value::Number)
    .unwrap_or(Value::Null)
}

fn bytes_to_bigint_string(bytes: &[u8]) -> String {
  if bytes.is_empty() {
    return "0".to_string();
  }
  let is_negative = (bytes[0] & 0x80) != 0;
  if is_negative {
    let mut inverted: Vec<u8> = bytes.iter().map(|b| !b).collect();
    let mut carry = 1u16;
    for byte in inverted.iter_mut().rev() {
      let sum = *byte as u16 + carry;
      *byte = sum as u8;
      carry = sum >> 8;
    }
    let magnitude = inverted.iter().fold(0u128, |acc, &b| (acc << 8) | b as u128);
    format!("-{}", magnitude)
  } else {
    let magnitude = bytes.iter().fold(0u128, |acc, &b| (acc << 8) | b as u128);
    format!("{}", magnitude)
  }
}

pub fn json_to_bind_value(value: &Value) -> Result<Option<CqlValue>, napi::Error> {
  match value {
    Value::Null => Ok(None),
    Value::Bool(boolean) => Ok(Some(CqlValue::Boolean(*boolean))),
    Value::Number(number) => {
      // Check f64 first: NAPI/JSON often stores large integers only as floats.
      if let Some(float) = number.as_f64() {
        if float.is_finite() && float.fract() == 0.0 {
          let int = float as i64;
          if int >= i32::MIN as i64 && int <= i32::MAX as i64 {
            return Ok(Some(CqlValue::Int(int as i32)));
          }
          return Ok(Some(CqlValue::BigInt(int)));
        }
        return Ok(Some(CqlValue::Double(float)));
      }

      if let Some(integer) = number.as_i64() {
        if integer >= i32::MIN as i64 && integer <= i32::MAX as i64 {
          return Ok(Some(CqlValue::Int(integer as i32)));
        }
        return Ok(Some(CqlValue::BigInt(integer)));
      }

      if let Some(unsigned) = number.as_u64() {
        if unsigned <= i32::MAX as u64 {
          return Ok(Some(CqlValue::Int(unsigned as i32)));
        }
        return Ok(Some(CqlValue::BigInt(unsigned as i64)));
      }

      Err(driver_error("Unsupported numeric bind value"))
    }
    Value::String(text) => {
      if let Ok(uuid) = Uuid::parse_str(text) {
        return Ok(Some(CqlValue::Uuid(uuid)));
      }
      // Try parsing as IP address for inet type
      if let Ok(ip) = text.parse::<IpAddr>() {
        return Ok(Some(CqlValue::Inet(ip)));
      }
      Ok(Some(CqlValue::Text(text.clone())))
    }
    Value::Array(values) => {
      // Check if it looks like a byte array (for blob): all numbers 0-255
      if !values.is_empty()
        && values.iter().all(
          |v| matches!(v, Value::Number(n) if n.as_u64().map(|b| b <= 255).unwrap_or(false)),
        )
      {
        let bytes: Vec<u8> = values.iter().map(|v| v.as_u64().unwrap() as u8).collect();
        return Ok(Some(CqlValue::Blob(bytes)));
      }
      // Otherwise treat as a list of CQL values
      let list: Vec<CqlValue> = values
        .iter()
        .map(|v| {
          json_to_bind_value(v)
            .and_then(|opt| opt.ok_or_else(|| driver_error("null not allowed in list elements")))
        })
        .collect::<Result<_, _>>()?;
      Ok(Some(CqlValue::List(list)))
    }
    Value::Object(map) => {
      // Check for float hint { __float: 3.14 }
      if let Some(Value::Number(n)) = map.get("__float") {
        if let Some(f) = n.as_f64() {
          return Ok(Some(CqlValue::Float(f as f32)));
        }
      }

      // Check for Buffer type { type: "Buffer", data: [...] }
      if let Some(Value::String(t)) = map.get("type") {
        if t == "Buffer" {
          if let Some(Value::Array(data)) = map.get("data") {
            let bytes: Result<Vec<u8>, _> = data
              .iter()
              .map(|v| {
                v.as_u64()
                  .and_then(|b| u8::try_from(b).ok())
                  .ok_or_else(|| driver_error("Buffer data must be bytes 0-255"))
              })
              .collect();
            return Ok(Some(CqlValue::Blob(bytes?)));
          }
        }
      }

      // Check for tuple { __tuple: [...] }
      if let Some(Value::Array(tuple_values)) = map.get("__tuple") {
        let tuple: Vec<Option<CqlValue>> = tuple_values
          .iter()
          .map(json_to_bind_value)
          .collect::<Result<_, _>>()?;
        return Ok(Some(CqlValue::Tuple(tuple)));
      }

      // Check for set { __set: [...] }
      if let Some(Value::Array(set_values)) = map.get("__set") {
        let set: Vec<CqlValue> = set_values
          .iter()
          .map(|v| {
            json_to_bind_value(v)
              .and_then(|opt| opt.ok_or_else(|| driver_error("null not allowed in set elements")))
          })
          .collect::<Result<_, _>>()?;
        return Ok(Some(CqlValue::Set(set)));
      }

      // Check for UDT { __udt_name: "...", __udt_keyspace: "...", field1: ..., field2: ... }
      if map.contains_key("__udt_name") {
        let name = map
          .get("__udt_name")
          .and_then(|v| v.as_str())
          .unwrap_or("")
          .to_string();
        let keyspace = map
          .get("__udt_keyspace")
          .and_then(|v| v.as_str())
          .unwrap_or("")
          .to_string();
        let mut fields: Vec<(String, Option<CqlValue>)> = Vec::new();
        for (key, val) in map.iter() {
          if key == "__udt_name" || key == "__udt_keyspace" {
            continue;
          }
          fields.push((key.clone(), json_to_bind_value(val)?));
        }
        return Ok(Some(CqlValue::UserDefinedType {
          name,
          keyspace,
          fields,
        }));
      }

      // Otherwise treat as a CQL map (string keys -> values)
      let entries: Vec<(CqlValue, CqlValue)> = map
        .iter()
        .map(|(k, v)| {
          let key = CqlValue::Text(k.clone());
          let val = json_to_bind_value(v)?
            .ok_or_else(|| driver_error("null not allowed as map value"))?;
          Ok((key, val))
        })
        .collect::<Result<_, napi::Error>>()?;
      Ok(Some(CqlValue::Map(entries)))
    }
  }
}
