use std::collections::HashMap;

use serde::{Deserialize, Deserializer};

use crate::input::Input;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Integer(i32),
    Float(f64),
    String(String),
}

impl Value {
    pub fn to_i32(&self) -> i32 {
        match self {
            Value::Integer(integer) => *integer,
            Value::Float(float) => *float as i32,
            Value::String(string) => string.parse::<f64>().unwrap_or(0.0) as i32,
        }
    }

    pub fn to_f64(&self) -> f64 {
        match self {
            Value::Integer(integer) => *integer as f64,
            Value::Float(float) => *float,
            Value::String(string) => string.parse::<f64>().unwrap_or(0.0),
        }
    }
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub value: Value,
}

/* I did not write this */
impl<'de> Deserialize<'de> for Variable {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let (name, value) = Deserialize::deserialize(de)?;
        Ok(Self { name, value })
    }
}

#[derive(Debug)]
pub struct List {
    pub name: String,
    pub value: Vec<Value>,
}

/* I did not write this */
impl<'de> Deserialize<'de> for List {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let (name, value) = Deserialize::deserialize(de)?;
        Ok(Self { name, value })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub opcode: String,
    pub next: Option<String>,
    pub parent: Option<String>,
    pub inputs: HashMap<String, Input>,
    //pub fields: HashMap<String, Field>,
    pub top_level: bool,
}
