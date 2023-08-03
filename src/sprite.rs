use serde::Deserialize;
use std::collections::HashMap;

use crate::{
    block::{Block, List, Value, Variable},
    costume::Costume,
    input::Input,
    script::Script,
};

#[derive(Debug, Deserialize)]
pub struct Sprite {
    #[serde(flatten)]
    pub data: SpriteData,
    #[serde(flatten)]
    pub state: SpriteState,
    #[serde(skip_deserializing)]
    pub scripts: Vec<Script>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteState {
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
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteData {
    pub name: String,
    pub blocks: HashMap<String, Block>,
    pub costumes: Vec<Costume>,
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

impl Sprite {
    fn evaluate_block(data: &SpriteData, state: &mut SpriteState, id: &str) -> Value {
        let block = &data.blocks[id];
        Value::Integer(0)
    }

    fn execute_block(data: &SpriteData, state: &mut SpriteState, id: &str) {
        let block = &data.blocks[id];
        match block.opcode.as_str() {
            "motion_goto" => {
                state.x = match &block.inputs["x"] {
                    Input::Block(block) => Sprite::evaluate_block(data, state, &block).to_i32(),
                    Input::Value(value) => value.to_i32(),
                    _ => panic!(),
                };
                state.y = match &block.inputs["y"] {
                    Input::Value(value) => value.to_i32(),
                    _ => panic!(),
                };
            }
            _ => panic!(),
        }
    }

    // Thanks to the guys at the Rust Discord Server.
    pub fn step_scripts(&mut self) {
        for script in &mut self.scripts {
            Sprite::step_script(&self.data, &mut self.state, script);
        }
    }

    fn step_script(data: &SpriteData, state: &mut SpriteState, script: &Script) {
        Sprite::execute_block(data, state, &script.id);
    }
}
