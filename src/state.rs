use sdl2::render::Texture;
use serde::{de::Visitor, Deserializer};
use serde::{
    de::{Error, SeqAccess},
    Deserialize,
};
use std::fmt;
use std::{collections::HashMap, fs::File, io::BufReader};

pub struct State<'a> {
    pub stage: Stage,
    pub sprites: Vec<Sprite<'a>>,
    pub stage_width: u32,
    pub stage_height: u32,
    pub frame_rate: u32,
    pub textures: HashMap<String, Texture<'a>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stage {
    pub current_costume: usize,
    pub volume: u8,
    pub variables: HashMap<String, Variable>,
    pub lists: HashMap<String, List>,
    pub blocks: HashMap<String, Block>,
    pub costumes: Vec<Costume>,
    pub tempo: u32,
    pub video_transparency: u8,
    pub video_state: VideoState,
    pub text_to_speech_language: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sprite<'a> {
    pub name: String,
    pub layer_order: usize,
    pub visible: bool,
    pub x: i32,
    pub y: i32,
    pub size: u32,
    pub direction: f64,
    pub draggable: bool,
    pub current_costume: usize,
    pub rotation_style: RotationStyle,
    pub volume: u8,
    pub variables: HashMap<String, Variable>,
    pub lists: HashMap<String, List>,
    pub blocks: HashMap<String, Block>,
    pub costumes: Vec<Costume>,
    #[serde(skip_deserializing)]
    pub scripts: Vec<Script<'a>>,
}

#[derive(Debug)]
pub struct Script<'a> {
    pub id: &'a String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Costume {
    pub name: String,
    pub bitmap_resolution: u32,
    pub md5ext: String,
    pub rotation_center_x: i32,
    pub rotation_center_y: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RotationStyle {
    #[serde(rename = "all around")]
    AllAround,
    #[serde(rename = "left-right")]
    LeftRight,
    #[serde(rename = "don't rotate")]
    DontRotate,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoState {
    On,
    Off,
    KillingTheRadioStar,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Integer(i32),
    Float(f64),
    String(String),
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub value: Value,
}

/* I did not write this */
impl<'a> Deserialize<'a> for Variable {
    fn deserialize<D: Deserializer<'a>>(de: D) -> Result<Self, D::Error> {
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
impl<'a> Deserialize<'a> for List {
    fn deserialize<D: Deserializer<'a>>(de: D) -> Result<Self, D::Error> {
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

pub fn load_virtual_machine_state() -> State<'static> {
    let project: Project =
        serde_json::from_reader(BufReader::new(File::open("project.json").unwrap())).unwrap();
    State {
        stage: project.targets.stage,
        sprites: project.targets.sprites,
        stage_width: 480,
        stage_height: 360,
        frame_rate: 30,
        textures: HashMap::new(),
    }
}

#[derive(Debug, Deserialize)]
pub struct Project<'a> {
    pub targets: TargetList<'a>,
}

#[derive(Debug)]
pub struct TargetList<'a> {
    pub stage: Stage,
    pub sprites: Vec<Sprite<'a>>,
}

/* I did not write this */
impl<'a> Deserialize<'a> for TargetList<'a> {
    fn deserialize<D: Deserializer<'a>>(de: D) -> Result<Self, D::Error> {
        struct TargetListVisitor;

        impl<'a> Visitor<'a> for TargetListVisitor {
            type Value = TargetList<'a>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "a sequence of a Stage followed by any number of Sprites")
            }

            fn visit_seq<A: SeqAccess<'a>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let stage = seq
                    .next_element()?
                    .ok_or_else(|| A::Error::missing_field("stage"))?;

                let mut sprites = Vec::new();
                while let Some(sprite) = seq.next_element()? {
                    sprites.push(sprite);
                }
                Ok(TargetList { stage, sprites })
            }
        }

        de.deserialize_seq(TargetListVisitor)
    }
}

#[derive(Debug)]
pub enum Input {
    Block(String),
    Value(Value),
    Broadcast(InputBroadcast),
    Variable(InputVariable),
    List(InputList),
}

/* I wrote this */
impl<'a> Deserialize<'a> for Input {
    fn deserialize<D: Deserializer<'a>>(de: D) -> Result<Self, D::Error> {
        struct SeqVisitor;
        impl<'a> Visitor<'a> for SeqVisitor {
            type Value = Input;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "Input")
            }
            fn visit_seq<A: SeqAccess<'a>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
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
