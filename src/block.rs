use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Value {
  Float(f64),
  String(String),
}

impl Value {
  pub fn to_i32(&self) -> i32 {
    match self {
      Value::Float(float) => *float as i32,
      Value::String(string) => string.parse::<f64>().unwrap_or(0.0) as i32,
    }
  }

  pub fn to_u32(&self) -> u32 {
    match self {
      Value::Float(float) => *float as u32,
      Value::String(string) => string.parse::<f64>().unwrap_or(0.0) as u32,
    }
  }

  pub fn to_f64(&self) -> f64 {
    match self {
      Value::Float(float) => *float,
      Value::String(string) => string.parse::<f64>().unwrap_or(0.0),
    }
  }

  pub fn map_as_str<T, F: FnOnce(&str) -> T>(&self, map: F) -> T {
    match self {
      Value::Float(float) => map(float.to_string().as_str()),
      Value::String(string) => map(string.as_str()),
    }
  }
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
  pub parent: usize,
  pub inputs: HashMap<String, Input>,
}
