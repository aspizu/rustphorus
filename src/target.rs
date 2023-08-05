use crate::{
  block::{Block, Input, Value},
  project::{Config, Texture},
  script::Script,
};
use derivative::Derivative;
use sdl2::{
  pixels::Color,
  rect::Rect,
  render::{Canvas, Texture as sdl2Texture, TextureCreator},
  ttf::Font,
  video::{Window, WindowContext},
};
use std::{collections::HashMap, f64::consts::PI};

#[derive(Debug)]
pub struct Target<'a> {
  pub data: TargetData,
  pub state: TargetState<'a>,
  pub scripts: Vec<Script>,
}

impl<'a> Target<'a> {
  pub fn render(
    data: &TargetData,
    state: &mut TargetState<'a>,
    textures: &Vec<Texture>,
    canvas: &mut Canvas<Window>,
    texture_creator: &'a TextureCreator<WindowContext>,
    font: &Font,
    config: &Config,
  ) {
    let texture =
      &textures[data.costume_index_to_texture_index[&state.current_costume]];
    let query = texture.texture.query();
    let scale = state.size / texture.bitmap_resolution as f64;
    let width = query.width as f64 * scale / 100.;
    let height = query.height as f64 * scale / 100.;
    let x = config.stage_width as i32 / 2 + state.x as i32 - width as i32 / 2;
    let y = config.stage_height as i32 / 2 + state.y as i32 - height as i32 / 2;
    let angle: f64;
    let flip: bool;
    match state.rotation_style {
      RotationStyle::AllAround => {
        angle = state.direction - 90.;
        flip = false;
      }
      RotationStyle::DontRotate => {
        angle = 0.;
        flip = false;
      }
      RotationStyle::LeftRight => {
        angle = 0.;
        flip = state.direction < 0.;
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
    if let Some(ref mut say) = &mut state.say {
      if say.texture.is_none() {
        say.texture = Some(
          texture_creator
            .create_texture_from_surface(
              font
                .render(say.message.as_str())
                .blended(Color::BLACK)
                .unwrap(),
            )
            .unwrap(),
        );
      }
      if let Some(texture) = &say.texture {
        let query = texture.query();
        canvas
          .copy(texture, None, Rect::new(x, y, query.width, query.height))
          .unwrap();
      }
    }
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
    "motion_setx" => state.x = aux_f64(data, state, &block.inputs["X"]),
    "motion_pointindirection" => {
      state.direction = aux_f64(data, state, &block.inputs["DIRECTION"])
    }
    "looks_say" => {
      state.say = Some(Say {
        message: aux_string(data, state, &block.inputs["MESSAGE"]).to_string(),
        texture: None,
      });
    }
    _ => panic!("I don't know how to execute: {block:#?}"),
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
    "operator_subtract" => Value::Float(
      aux_f64(data, state, &block.inputs["NUM1"])
        - aux_f64(data, state, &block.inputs["NUM2"]),
    ),
    "operator_multiply" => Value::Float(
      aux_f64(data, state, &block.inputs["NUM1"])
        * aux_f64(data, state, &block.inputs["NUM2"]),
    ),
    "operator_divide" => Value::Float(
      aux_f64(data, state, &block.inputs["NUM1"])
        / aux_f64(data, state, &block.inputs["NUM2"]),
    ),
    "operator_equals" => Value::Bool(
      aux_value(data, state, &block.inputs["OPERAND1"]).compare(&aux_value(
        data,
        state,
        &block.inputs["OPERAND2"],
      )) == 0.,
    ),
    "operator_gt" => Value::Bool(
      aux_value(data, state, &block.inputs["OPERAND1"]).compare(&aux_value(
        data,
        state,
        &block.inputs["OPERAND2"],
      )) > 0.,
    ),
    "operator_lt" => Value::Bool(
      aux_value(data, state, &block.inputs["OPERAND1"]).compare(&aux_value(
        data,
        state,
        &block.inputs["OPERAND2"],
      )) < 0.,
    ),
    "operator_letter_of" => {
      Value::String(aux_map_as_str(data, state, &block.inputs["STRING"], |s| {
        s.chars()
          .nth(aux_f64(data, state, &block.inputs["LETTER"]) as usize - 1)
          .and_then(|c| Some(c.to_string()))
          .unwrap_or(format!(""))
      }))
    }
    "operator_and" => Value::Bool(
      aux_bool(data, state, &block.inputs["OPERAND1"])
        && aux_bool(data, state, &block.inputs["OPERAND2"]),
    ),
    "operator_or" => Value::Bool(
      aux_bool(data, state, &block.inputs["OPERAND1"])
        || aux_bool(data, state, &block.inputs["OPERAND2"]),
    ),
    "operator_not" => Value::Bool(!aux_bool(data, state, &block.inputs["OPERAND"])),
    "operator_random" => Value::Float({
      let from = aux_value(data, state, &block.inputs["FROM"]);
      let to = aux_value(data, state, &block.inputs["TO"]);
      let n_from = from.to_f64();
      let n_to = to.to_f64();
      let (low, high) = if n_from <= n_to {
        (n_from, n_to)
      } else {
        (n_to, n_from)
      };
      if low == high {
        low
      } else if from.is_int() && to.is_int() {
        unsafe { (low as i32 + libc::rand() % (high as i32 + 1)) as f64 }
      } else {
        unsafe { low + (libc::rand() as f64 / i32::MAX as f64) * (high - low) }
      }
    }),
    "operator_join" => Value::String(aux_map_as_str(
      data,
      state,
      &block.inputs["STRING1"],
      |s1| {
        aux_map_as_str(data, state, &block.inputs["STRING2"], |s2| {
          format!("{s1}{s2}")
        })
      },
    )),
    "operator_length" => {
      Value::Float(
        aux_map_as_str(data, state, &block.inputs["STRING"], |s| s.len()) as f64,
      )
    }
    "operator_contains" => Value::Bool(aux_map_as_str(
      data,
      state,
      &block.inputs["STRING1"],
      |s1| {
        aux_map_as_str(data, state, &block.inputs["STRING2"], |s2| {
          s1.to_lowercase().contains(s2.to_lowercase().as_str())
        })
      },
    )),
    "operator_mod" => Value::Float({
      let n = aux_f64(data, state, &block.inputs["NUM1"]);
      let modulus = aux_f64(data, state, &block.inputs["NUM2"]);
      let mut result = n % modulus;
      if result / modulus < 0. {
        result += modulus;
      }
      result
    }),
    "operator_round" => {
      Value::Float(aux_f64(data, state, &block.inputs["NUM"]).round())
    }
    "operator_mathop" => {
      let value = aux_f64(data, state, &block.inputs["NUM"]);
      Value::Float(aux_field(block, "OPERATOR", |operator| match operator {
        "abs" => value.abs(),
        "floor" => value.floor(),
        "ceiling" => value.ceil(),
        "sqrt" => value.sqrt(),
        "sin" => truncate_float(degrees_to_radians(value).sin()),
        "cos" => truncate_float(degrees_to_radians(value).cos()),
        "tan" => {
          let angle = value % 360.;
          if angle == -270. || angle == 90. {
            f64::INFINITY
          } else if angle == -90. || angle == 270. {
            f64::NEG_INFINITY
          } else {
            truncate_float(degrees_to_radians(angle).tan())
          }
        }
        "asin" => radians_to_degrees(value.asin()),
        "acos" => radians_to_degrees(value.acos()),
        "atan" => radians_to_degrees(value.atan()),
        "ln" => value.ln(),
        "log" => value.log10(),
        "e ^" => value.exp(),
        "10 ^" => value.powi(10),
        s => panic!("I don't know how to perform: {s}"),
      }))
    }
    "motion_xposition" => Value::Float(limit_precision(state.x)),
    "motion_yposition" => Value::Float(limit_precision(state.y)),
    "motion_direction" => Value::Float(state.direction),
    "looks_costumenumbername" => aux_field(block, "NUMBER_NAME", |s| match s {
      "number" => Value::Float(1. + state.current_costume as f64),
      "name" => {
        Value::String(data.costume_index_to_name[state.current_costume].clone())
      }
      _ => panic!(),
    }),
    _ => {
      panic!("I don't know how to evaluate: {block:#?}")
    }
  }
}

fn limit_precision(value: f64) -> f64 {
  let rounded = value.round();
  if (value - rounded).abs() < 1e-9_f64 {
    rounded
  } else {
    value
  }
}

fn degrees_to_radians(degrees: f64) -> f64 {
  (PI * degrees) / 180.
}

fn radians_to_degrees(degrees: f64) -> f64 {
  (degrees * 180.) / PI
}

fn truncate_float(value: f64) -> f64 {
  format!("{:.10}", value).parse().unwrap()
}

fn aux_f64(data: &TargetData, state: &TargetState, input: &Input) -> f64 {
  match input {
    Input::Block(id) => evaluate_block(data, state, *id).to_f64(),
    Input::Value(value) => value.to_f64(),
    Input::Variable(variable) => state.variables[variable.id].to_f64(),
    _ => 0.,
  }
}

fn aux_bool(data: &TargetData, state: &TargetState, input: &Input) -> bool {
  match input {
    Input::Block(id) => evaluate_block(data, state, *id).to_bool(),
    Input::Value(value) => value.to_bool(),
    Input::Variable(variable) => state.variables[variable.id].to_bool(),
    _ => false,
  }
}

fn aux_string(data: &TargetData, state: &TargetState, input: &Input) -> String {
  match input {
    Input::Block(id) => evaluate_block(data, state, *id).to_string(),
    Input::Value(value) => value.to_string(),
    Input::Variable(variable) => state.variables[variable.id].to_string(),
    _ => panic!(),
  }
}

fn aux_value(data: &TargetData, state: &TargetState, input: &Input) -> Value {
  match input {
    Input::Block(id) => evaluate_block(data, state, *id),
    Input::Value(value) => value.clone(),
    Input::Variable(variable) => state.variables[variable.id].clone(),
    _ => panic!(),
  }
}

fn aux_field<T, F: FnOnce(&str) -> T>(block: &Block, name: &str, map: F) -> T {
  match &block.inputs[name] {
    Input::Value(value) => match value {
      Value::String(value) => map(value.as_str()),
      _ => panic!(),
    },
    _ => panic!(),
  }
}

fn aux_map_as_str<T, F: FnOnce(&str) -> T>(
  data: &TargetData,
  state: &TargetState,
  input: &Input,
  map: F,
) -> T {
  match input {
    Input::Block(id) => evaluate_block(data, state, *id).map_as_str(map),
    Input::Value(value) => value.map_as_str(map),
    Input::Variable(variable) => state.variables[variable.id].map_as_str(map),
    _ => panic!(),
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
  pub costume_index_to_name: Vec<String>,
  pub costume_name_to_index: HashMap<String, usize>,
  pub costume_index_to_texture_index: HashMap<usize, usize>,
}

#[derive(Debug)]
pub struct TargetState<'a> {
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
  pub say: Option<Say<'a>>,
}

#[derive(Debug)]
pub enum RotationStyle {
  AllAround,
  LeftRight,
  DontRotate,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Say<'a> {
  pub message: String,
  #[derivative(Debug = "ignore")]
  pub texture: Option<sdl2Texture<'a>>,
}
