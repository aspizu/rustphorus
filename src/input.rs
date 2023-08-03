use serde::de::SeqAccess;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Deserializer;
use std::fmt;
use std::fmt::Formatter;

use crate::block::Value;

#[derive(Debug)]
pub enum Input {
    Block(String),
    Value(Value),
    Broadcast(InputBroadcast),
    Variable(InputVariable),
    List(InputList),
}

/* I wrote this */
impl<'de> Deserialize<'de> for Input {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct SeqVisitor;
        impl<'de> Visitor<'de> for SeqVisitor {
            type Value = Input;
            fn expecting(&self, f: &mut Formatter) -> fmt::Result {
                write!(f, "Input")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                #[derive(Debug, Deserialize)]
                #[serde(untagged)]
                enum T {
                    String(String),
                    Values(Vec<Value>),
                }
                let _shadow = seq.next_element::<i32>()?;
                match seq.next_element::<T>()? {
                    Some(T::String(string)) => {
                        while seq.next_element::<serde_json::Value>()?.is_some() {}
                        return Ok(Input::Block(string));
                    }
                    Some(T::Values(mut values)) => match values[0] {
                        Value::Integer(4)
                        | Value::Integer(5)
                        | Value::Integer(6)
                        | Value::Integer(7)
                        | Value::Integer(8)
                        | Value::Integer(9)
                        | Value::Integer(10) => {
                            return Ok(Input::Value(values.remove(1)));
                        }
                        Value::Integer(11) => {
                            return Ok(Input::Broadcast(InputBroadcast {
                                name: match values.remove(1) {
                                    Value::String(string) => string,
                                    _ => panic!(),
                                },
                                id: match values.remove(1) {
                                    Value::String(string) => string,
                                    _ => panic!(),
                                },
                            }));
                        }
                        Value::Integer(12) => {
                            return Ok(Input::Variable(InputVariable {
                                name: match values.remove(1) {
                                    Value::String(string) => string,
                                    _ => panic!(),
                                },
                                id: match values.remove(1) {
                                    Value::String(string) => string,
                                    _ => panic!(),
                                },
                            }));
                        }
                        Value::Integer(13) => {
                            return Ok(Input::List(InputList {
                                name: match values.remove(1) {
                                    Value::String(string) => string,
                                    _ => panic!(),
                                },
                                id: match values.remove(1) {
                                    Value::String(string) => string,
                                    _ => panic!(),
                                },
                            }));
                        }
                        _ => todo!(),
                    },
                    None => todo!(),
                }
            }
        }
        de.deserialize_seq(SeqVisitor)
    }
}

#[derive(Debug)]
pub struct InputBroadcast {
    pub name: String,
    pub id: String,
}

#[derive(Debug)]
pub struct InputVariable {
    pub name: String,
    pub id: String,
}

#[derive(Debug)]
pub struct InputList {
    pub name: String,
    pub id: String,
}
