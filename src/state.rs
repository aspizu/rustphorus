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
    pub sprites: Vec<Sprite>,
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
pub struct Sprite {
    pub name: String,
    pub layer_order: usize,
    pub visible: bool,
    pub x: i32,
    pub y: i32,
    pub size: u32,
    pub direction: i8,
    pub draggable: bool,
    pub current_costume: usize,
    pub rotation_style: RotationStyle,
    pub volume: u8,
    pub variables: HashMap<String, Variable>,
    pub lists: HashMap<String, List>,
    pub blocks: HashMap<String, Block>,
    pub costumes: Vec<Costume>,
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
pub enum Value {
    Text(String),
    Integer(i32),
    Float(f64),
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
    pub inputs: HashMap<String, BlockInput>,
    pub fields: HashMap<String, BlockField>,
    pub top_level: bool,
}

#[derive(Debug, Deserialize)]
pub struct BlockInput {}

#[derive(Debug, Deserialize)]
pub struct BlockField {}

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
pub struct Project {
    pub targets: TargetList,
}

#[derive(Debug)]
pub struct TargetList {
    pub stage: Stage,
    pub sprites: Vec<Sprite>,
}

/* I did not write this */
impl<'de> Deserialize<'de> for TargetList {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct TargetListVisitor;

        impl<'de> Visitor<'de> for TargetListVisitor {
            type Value = TargetList;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a sequence of a Stage followed by any number of Sprites")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
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
