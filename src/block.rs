use serde::Deserialize;
use std::collections::HashMap;

fn compare_string(s1: &str, s2: &str) -> f64 {
  let s1 = s1.to_lowercase();
  let s2 = s2.to_lowercase();
  match s1.cmp(&s2) {
    std::cmp::Ordering::Less => -1.,
    std::cmp::Ordering::Equal => 0.,
    std::cmp::Ordering::Greater => 1.,
  }
}

fn map_f64_as_str<T, F: FnOnce(&str) -> T>(value: f64, map: F) -> T {
  if value.is_infinite() {
    if value.is_sign_positive() {
      map("Infinity")
    } else {
      map("-Infinity")
    }
  } else if value.is_nan() {
    map("NaN")
  } else if value == 0. && value.is_sign_negative() {
    map("0")
  } else {
    map(value.to_string().as_str())
  }
}

fn parse_number(s: &str) -> f64 {
  let s = s.trim();
  match s {
    "inf" => 0.,
    "-inf" => 0.,
    "nan" => 0.,
    "infinity" => 0.,
    "-infinity" => 0.,
    "Infinity" => f64::INFINITY,
    "-Infinity" => f64::NEG_INFINITY,
    "NaN" => f64::NAN,
    s => s.parse::<f64>().unwrap_or(0.),
  }
}

impl Value {
  pub fn is_int(&self) -> bool {
    match self {
      Value::Bool(_) => true,
      Value::Float(float) => float.round() == *float,
      Value::String(string) => !string.contains('.'),
    }
  }

  pub fn to_f64(&self) -> f64 {
    match self {
      Value::Float(float) => {
        if float.is_nan() {
          0.
        } else {
          *float
        }
      }
      Value::String(string) => parse_number(string.as_str()),
      Value::Bool(bool) => {
        if *bool {
          1.
        } else {
          0.
        }
      }
    }
  }

  pub fn to_bool(&self) -> bool {
    match self {
      Value::Bool(bool) => *bool,
      Value::Float(float) => *float != 0.,
      Value::String(string) => match string.to_lowercase().as_str() {
        "" | "0" | "false" => false,
        _ => true,
      },
    }
  }

  pub fn is_whitespace(&self) -> bool {
    match self {
      Value::String(string) => string.trim().len() == 0,
      _ => false,
    }
  }

  pub fn map_as_str<T, F: FnOnce(&str) -> T>(&self, map: F) -> T {
    match self {
      Value::Float(float) => map_f64_as_str(*float, map),
      Value::String(string) => map(string.as_str()),
      Value::Bool(bool) => {
        if *bool {
          map("true")
        } else {
          map("false")
        }
      }
    }
  }

  pub fn to_string(&self) -> String {
    match self {
      Value::Float(float) => map_f64_as_str(*float, |s| String::from(s)),
      Value::String(string) => string.clone(),
      Value::Bool(bool) => {
        if *bool {
          format!("true")
        } else {
          format!("false")
        }
      }
    }
  }

  pub fn compare(&self, other: &Value) -> f64 {
    match self {
      Value::Float(n1) => match other {
        Value::Float(n2) => n1 - n2,
        Value::Bool(n2) => n1 - (*n2 as i32 as f64),
        Value::String(n2) => n2
          .parse::<f64>()
          .and_then(|n2| Ok(n1 - n2))
          .unwrap_or(compare_string(n1.to_string().as_str(), n2.as_str())),
      },
      Value::Bool(n1) => match other {
        Value::Float(n2) => (*n1 as i32 as f64) - *n2,
        Value::Bool(n2) => (*n1 as i32 as f64) - (*n2 as i32 as f64),
        Value::String(n2) => n2
          .parse::<f64>()
          .and_then(|n2| Ok((*n1 as i32 as f64) - n2))
          .unwrap_or(compare_string(n1.to_string().as_str(), n2.as_str())),
      },
      Value::String(n1) => match other {
        Value::Float(n2) => n1
          .parse::<f64>()
          .and_then(|n1| Ok(n1 - *n2))
          .unwrap_or(compare_string(n1.as_str(), n2.to_string().as_str())),
        Value::Bool(n2) => n1
          .parse::<f64>()
          .and_then(|n1| Ok(n1 - (*n2 as i32 as f64)))
          .unwrap_or(compare_string(n1.as_str(), n2.to_string().as_str())),
        Value::String(n2) => compare_string(n1.as_str(), n2.as_str()),
      },
    }
  }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Value {
  Float(f64),
  String(String),
  Bool(bool),
}

#[derive(Debug)]
pub enum Input {
  Block(usize),
  Value(Value),
  Broadcast(BroadcastInput),
  Variable(VariableInput),
  List(ListInput),
}

#[derive(Debug)]
pub struct BroadcastInput {
  pub name: String,
  pub id: String,
}

#[derive(Debug)]
pub struct VariableInput {
  pub is_global: bool,
  pub id: usize,
}

#[derive(Debug)]
pub struct ListInput {
  pub is_global: bool,
  pub id: usize,
}

#[derive(Debug)]
pub struct Block {
  pub opcode: String,
  pub next: usize,
  // pub parent: usize,
  pub inputs: HashMap<String, Input>,
}

#[derive(Debug)]
pub struct CustomBlock {
  pub next: usize,
  pub argument_ids: Vec<String>,
  pub refresh: bool,
}
