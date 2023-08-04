use crate::{
  block::{Block, Input, Value},
  project::{Config, Texture},
  script::Script,
};
use sdl2::{rect::Rect, render::Canvas, video::Window};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Target {
  pub data: TargetData,
  pub state: TargetState,
  pub scripts: Vec<Script>,
}

impl Target {
  pub fn render(
    &self,
    textures: &Vec<Texture>,
    canvas: &mut Canvas<Window>,
    config: &Config,
  ) {
    let texture =
      &textures[self.data.costume_index_to_texture_index[&self.state.current_costume]];
    let query = texture.texture.query();
    let scale = self.state.size / texture.bitmap_resolution as f64;
    let width = query.width as f64 * scale / 100.;
    let height = query.height as f64 * scale / 100.;
    let x = config.stage_width as i32 / 2 + self.state.x as i32 - width as i32 / 2;
    let y = config.stage_height as i32 / 2 + self.state.y as i32 - height as i32 / 2;
    let angle: f64;
    let flip: bool;
    match self.state.rotation_style {
      RotationStyle::AllAround => {
        angle = self.state.direction - 90.;
        flip = false;
      }
      RotationStyle::DontRotate => {
        angle = 0.;
        flip = false;
      }
      RotationStyle::LeftRight => {
        angle = 0.;
        flip = self.state.direction < 0.;
      }
    }
    canvas
      .copy_ex(
        &texture.texture,
        None,
        Rect::new(x, y, width as u32, height as u32),
        angle,
        None,
        false,
        flip,
      )
      .unwrap();
  }

  pub fn start_scripts(&mut self) {
    for (index, block) in self.data.blocks.iter().enumerate() {
      match block.opcode.as_str() {
        "event_whenflagclicked" => self.scripts.push(Script { id: index + 1 }),
        _ => {}
      }
    }
  }

  pub fn execute_scripts(&mut self) {
    self.scripts.retain_mut(|script| {
      script.id = execute_block(&self.data, &mut self.state, script.id);
      script.id != 0
    });
  }
}

fn execute_block(data: &TargetData, state: &mut TargetState, id: usize) -> usize {
  let block = &data.blocks[id - 1];
  match block.opcode.as_str() {
    "event_whenflagclicked" => {}
    "motion_gotoxy" => {
      state.x = aux_f64(data, state, &block.inputs["X"]);
      state.y = aux_f64(data, state, &block.inputs["Y"]);
    }
    _ => {
      let opcode = &block.opcode;
      panic!("{opcode}")
    }
  }
  block.next
}

fn evaluate_block(data: &TargetData, state: &TargetState, id: usize) -> Value {
  let block = &data.blocks[id - 1];
  match block.opcode.as_str() {
    "operator_add" => Value::Float(
      aux_f64(data, state, &block.inputs["NUM1"])
        + aux_f64(data, state, &block.inputs["NUM2"]),
    ),
    _ => {
      let opcode = &block.opcode;
      panic!("{opcode}")
    }
  }
}

fn aux_i32(data: &TargetData, state: &TargetState, input: &Input) -> i32 {
  match input {
    Input::Block(id) => evaluate_block(data, state, *id).to_i32(),
    Input::Value(value) => value.to_i32(),
    Input::Variable(variable) => state.variables[variable.id].to_i32(),
    _ => 0,
  }
}

fn aux_f64(data: &TargetData, state: &TargetState, input: &Input) -> f64 {
  match input {
    Input::Block(id) => evaluate_block(data, state, *id).to_f64(),
    Input::Value(value) => value.to_f64(),
    Input::Variable(variable) => state.variables[variable.id].to_f64(),
    _ => 0.,
  }
}

fn aux_id(input: &Input) -> usize {
  match input {
    Input::Block(id) => *id,
    _ => panic!(),
  }
}

#[derive(Debug)]
pub struct TargetData {
  pub is_stage: bool,
  pub blocks: Vec<Block>,
  pub costume_name_to_index: HashMap<String, usize>,
  pub costume_index_to_texture_index: HashMap<usize, usize>,
}

#[derive(Debug)]
pub struct TargetState {
  pub visible: bool,
  pub x: f64,
  pub y: f64,
  pub size: f64,
  pub direction: f64,
  pub draggable: bool,
  pub current_costume: usize,
  pub rotation_style: RotationStyle,
  pub volume: f64,
  pub variables: Vec<Value>,
  pub lists: Vec<Vec<Value>>,
}

#[derive(Debug)]
pub enum RotationStyle {
  AllAround,
  LeftRight,
  DontRotate,
}
