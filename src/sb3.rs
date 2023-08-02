use std::{collections::HashMap, fs::File, io::BufReader};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub targets: Vec<Target>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub is_stage: bool,
    pub name: String,
    pub variables: HashMap<String, Variable>, // just guessing
    pub lists: HashMap<String, List>,
    pub broadcasts: HashMap<String, Broadcast>,
    pub blocks: HashMap<String, Block>,
    pub comments: HashMap<String, Comment>,
    pub current_costume: usize,
    pub costumes: Vec<Costume>,
    pub sounds: Vec<Sound>,
    pub volume: u8,
    pub layer_order: i32,
    #[serde(flatten)]
    pub details: Details,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Details {
    Stage(StageDetails),
    Sprite(SpriteDetails),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StageDetails {
    pub tempo: u32,
    pub video_transparency: u8,
    pub video_state: VideoState,
    pub text_to_speech_language: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteDetails {
    pub visible: bool,
    pub x: i32,
    pub y: i32,
    pub size: u32,
    pub direction: i16,
    pub draggable: bool,
    pub rotation_style: RotationStyle,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoState {
    On,
    Off,
    KillingTheRadioStar,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RotationStyle {
    #[serde(rename = "all around")]
    AllAround,
    #[serde(rename = "left-right")]
    LeftRight,
    #[serde(rename = "don't rotate")]
    DontRotate,
}

type Variable = ();
type List = ();
type Broadcast = ();
type Block = ();
type Comment = ();

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Costume {
    pub name: String,
    pub bitmap_resolution: u8,
    pub data_format: String,
    pub asset_id: String,
    pub md5ext: String,
    pub rotation_center_x: i32,
    pub rotation_center_y: i32,
}

type Sound = ();

pub fn load(path: &str) -> Project {
    return serde_json::from_reader(BufReader::new(File::open(path).unwrap())).unwrap();
}
