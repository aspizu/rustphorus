use crate::{
  block::{Block, CustomBlock, Input, Value, VariableInput},
  pen::PenInstruction,
  project::{Config, SharedState, Texture},
  script::{Script, StackFrame},
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

fn clamp_size(size: f64) -> f64 {
  if size < 0. {
    0.
  } else {
    size
  }
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
    if !state.visible {
      return;
    }
    let texture =
      &textures[data.costume_index_to_texture_index[&state.current_costume]];
    let query = texture.texture.query();
    state.size = clamp_size(state.size);
    let scale = state.size / texture.bitmap_resolution as f64;
    let width = query.width as f64 * scale / 100.;
    let height = query.height as f64 * scale / 100.;
    let x = config.stage_width as i32 / 2 + state.x as i32 - width as i32 / 2;
    let y = (config.stage_height as i32 / 2 - state.y as i32) - height as i32 / 2;
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
        "event_whenflagclicked" => self.scripts.push(Script {
          id: index + 1,
          stack: vec![],
          arguments: vec![],
          arguments_start: 0,
          refresh: true,
        }),
        _ => {}
      }
    }
  }

  pub fn execute_scripts(&mut self, shared: &mut SharedState) {
    self.scripts.retain_mut(|script| loop {
      let (terminate, refresh) =
        execute_script(shared, &self.data, &mut self.state, script);
      if terminate || script.refresh && refresh {
        return !terminate;
      }
    });
  }
}

/// Returns (should terminate, should refresh screen)
fn execute_script(
  shared: &mut SharedState,
  data: &TargetData,
  state: &mut TargetState,
  script: &mut Script,
) -> (bool, bool) {
  let mut terminate = false;
  let mut refresh = false;
  let block = &data.blocks[script.id - 1];
  log::trace!("{block:#?}");
  match block.opcode.as_str() {
    "control_repeat" => {
      let iterations =
        aux_f64(shared, data, state, &block.inputs["TIMES"], script) as u32;
      if iterations > 0 {
        let jump_id = aux_id(&block.inputs["SUBSTACK"]);
        script.stack.push(StackFrame::Repeat {
          iterations: iterations as u32,
          jump_id,
          return_id: block.next,
        });
        script.id = jump_id;
      }
    }
    "control_forever" => {
      let jump_id = aux_id(&block.inputs["SUBSTACK"]);
      script.stack.push(StackFrame::Goto(script.id));
      script.id = jump_id;
    }
    "control_if_else" => {
      if aux_bool(shared, data, state, &block.inputs["CONDITION"], script) {
        script.id = aux_id(&block.inputs["SUBSTACK"]);
      } else {
        script.id = aux_id(&block.inputs["SUBSTACK2"]);
      }
      script.stack.push(StackFrame::Goto(block.next));
    }
    "control_if" => {
      if aux_bool(shared, data, state, &block.inputs["CONDITION"], script) {
        script.id = aux_id(&block.inputs["SUBSTACK"]);
        script.stack.push(StackFrame::Goto(block.next));
      } else {
        script.id = block.next;
      }
    }
    "procedures_call" => {
      log::trace!("{script:#?}");
      let custom_block = aux_field(block, "PROCCODE", |s| &data.custom_blocks[s]);
      script.stack.push(StackFrame::CustomBlock {
        argument_count: custom_block.argument_ids.len(),
        return_id: block.next,
        refresh_was_set_false: script.refresh && !custom_block.refresh,
        old_arguments_start: script.arguments_start,
      });
      let new_arguments_start = script.arguments.len();
      script.id = custom_block.next;
      for id in &custom_block.argument_ids {
        script.arguments.push(aux_value(
          shared,
          data,
          state,
          &block.inputs[id],
          script,
        ));
      }
      script.arguments_start = new_arguments_start;
      if script.refresh && !custom_block.refresh {
        script.refresh = false;
      }
    }
    _ => {
      refresh = execute_block(shared, data, state, script.id, &script);
      script.id = block.next;
    }
  }
  #[allow(unused_assignments)]
  let mut pop = false;
  while script.id == 0 {
    if script.stack.last().is_some() {
      log::trace!("{script:#?}");
    }
    if let Some(item) = script.stack.last_mut() {
      match item {
        StackFrame::Repeat {
          iterations,
          jump_id,
          return_id,
        } => {
          *iterations -= 1;
          if *iterations > 0 {
            script.id = *jump_id;
            break;
          } else {
            script.id = *return_id;
            pop = true;
          }
        }
        StackFrame::Goto(id) => {
          script.id = *id;
          pop = true;
        }
        StackFrame::CustomBlock {
          argument_count,
          return_id,
          refresh_was_set_false,
          old_arguments_start,
        } => {
          script.id = *return_id;
          if *refresh_was_set_false {
            script.refresh = true;
          }
          script
            .arguments
            .truncate(script.arguments.len() - *argument_count);
          script.arguments_start = *old_arguments_start;
          pop = true;
        }
      }
    } else {
      break;
    }
    if pop {
      script.stack.pop();
    }
  }
  if script.id == 0 {
    log::trace!("terminated");
    terminate = true;
  }
  return (terminate, refresh);
}

fn get_argument(index: usize, script: &Script) -> Value {
  log::trace!("({index}, {script:#?})");
  let peek = script.arguments_start + index;
  script.arguments[peek].clone()
}

fn get_direction(direction: f64) -> Option<f64> {
  if direction == 0. || direction.is_normal() {
    Some(wrap_clamp(direction, -179., 180.))
  } else {
    None
  }
}

fn wrap_clamp(value: f64, min: f64, max: f64) -> f64 {
  let range = (max - min) + 1.;
  value - ((value - min) / range).floor() * range
}

/// Returns true if screen should be refreshed
fn execute_block(
  shared: &mut SharedState,
  data: &TargetData,
  state: &mut TargetState,
  id: usize,
  script: &Script,
) -> bool {
  let mut refresh = false;
  let block = &data.blocks[id - 1];
  match block.opcode.as_str() {
    "event_whenflagclicked" => {}
    "motion_gotoxy" => {
      state.x = aux_f64(shared, data, state, &block.inputs["X"], script);
      state.y = aux_f64(shared, data, state, &block.inputs["Y"], script);
      refresh = true;
    }
    "motion_setx" => {
      state.x = aux_f64(shared, data, state, &block.inputs["X"], script);
      refresh = true;
    }
    "motion_sety" => {
      state.y = aux_f64(shared, data, state, &block.inputs["Y"], script);
      refresh = true;
    }
    "motion_changexby" => {
      state.x += aux_f64(shared, data, state, &block.inputs["DX"], script);
      refresh = true;
    }
    "motion_changeyby" => {
      state.y += aux_f64(shared, data, state, &block.inputs["DY"], script);
      refresh = true;
    }
    "motion_pointindirection" => {
      if let Some(direction) = get_direction(aux_f64(
        shared,
        data,
        state,
        &block.inputs["DIRECTION"],
        script,
      )) {
        log::trace!("direction: {direction}");
        state.direction = direction;
      }
      refresh = true;
    }
    "motion_turnright" => {
      if let Some(direction) = get_direction(
        state.direction
          + aux_f64(shared, data, state, &block.inputs["DEGREES"], script),
      ) {
        log::trace!("direction: {direction}");
        state.direction = direction;
      }
      refresh = true;
    }
    "motion_turnleft" => {
      if let Some(direction) = get_direction(
        state.direction
          - aux_f64(shared, data, state, &block.inputs["DEGREES"], script),
      ) {
        state.direction = direction;
      }
      refresh = true;
    }
    "looks_say" => {
      let message =
        aux_string(shared, data, state, &block.inputs["MESSAGE"], script).to_string();
      log::info!("{message}");
      state.say = if message.len() == 0 {
        None
      } else {
        Some(Say {
          message,
          texture: None,
        })
      };
      refresh = true;
    }
    "data_setvariableto" => {
      let Input::Variable(variable) = &block.inputs["VARIABLE"] else { panic!() };
      let value = aux_value(shared, data, state, &block.inputs["VALUE"], script);
      set_variable(shared, state, variable, |_| value);
    }
    "data_changevariableby" => {
      let Input::Variable(variable) = &block.inputs["VARIABLE"] else { panic!() };
      let change = aux_value(shared, data, state, &block.inputs["VALUE"], script);
      set_variable(shared, state, variable, |value| {
        Value::Float(value.to_f64() + change.to_f64())
      });
    }
    "data_deletealloflist" => {
      let Input::List(list) = &block.inputs["LIST"] else { panic!(); };
      if list.is_global {
        shared.global_lists[list.id].clear();
      } else {
        state.lists[list.id].clear();
      }
    }
    "data_addtolist" => {
      let Input::List(list) = &block.inputs["LIST"] else { panic!(); };
      let value = aux_value(shared, data, state, &block.inputs["ITEM"], script);
      if list.is_global {
        shared.global_lists[list.id].push(value);
      } else {
        state.lists[list.id].push(value);
      }
    }
    "looks_setsizeto" => {
      let size = aux_f64(shared, data, state, &block.inputs["SIZE"], script);
      state.size = size;
    }
    "pen_clear" => {
      shared.pen.clear();
    }
    "pen_setPenSizeTo" => {
      let size = aux_f64(shared, data, state, &block.inputs["SIZE"], script);
      if 0. < size {
        state.pen.size = size as u32;
      }
    }
    "pen_penDown" => {
      state.pen.is_down = true;
      state.pen.x = state.x;
      state.pen.y = state.y;
    }
    "pen_penUp" => {
      if state.pen.is_down {
        update_pen(shared, state);
      }
      state.pen.is_down = false;
    }
    _ => panic!("I don't know how to execute: {block:#?}"),
  }
  refresh
}

fn update_pen(shared: &mut SharedState, state: &mut TargetState) {
  shared.pen.push_back(PenInstruction {
    size: state.pen.size,
    r: state.pen.r,
    g: state.pen.g,
    b: state.pen.b,
    a: state.pen.a,
    x1: state.pen.x,
    y1: state.pen.y,
    x2: state.x,
    y2: state.y,
  });
  state.pen.x = state.x;
  state.pen.y = state.y;
}

fn evaluate_block(
  shared: &SharedState,
  data: &TargetData,
  state: &TargetState,
  id: usize,
  script: &Script,
) -> Value {
  let block = &data.blocks[id - 1];
  match block.opcode.as_str() {
    "operator_add" => Value::Float(
      aux_f64(shared, data, state, &block.inputs["NUM1"], script)
        + aux_f64(shared, data, state, &block.inputs["NUM2"], script),
    ),
    "operator_subtract" => Value::Float(
      aux_f64(shared, data, state, &block.inputs["NUM1"], script)
        - aux_f64(shared, data, state, &block.inputs["NUM2"], script),
    ),
    "operator_multiply" => Value::Float(
      aux_f64(shared, data, state, &block.inputs["NUM1"], script)
        * aux_f64(shared, data, state, &block.inputs["NUM2"], script),
    ),
    "operator_divide" => Value::Float(
      aux_f64(shared, data, state, &block.inputs["NUM1"], script)
        / aux_f64(shared, data, state, &block.inputs["NUM2"], script),
    ),
    "operator_equals" => Value::Bool(
      aux_value(shared, data, state, &block.inputs["OPERAND1"], script).compare(
        &aux_value(shared, data, state, &block.inputs["OPERAND2"], script),
      ) == 0.,
    ),
    "operator_gt" => Value::Bool(
      aux_value(shared, data, state, &block.inputs["OPERAND1"], script).compare(
        &aux_value(shared, data, state, &block.inputs["OPERAND2"], script),
      ) > 0.,
    ),
    "operator_lt" => Value::Bool(
      aux_value(shared, data, state, &block.inputs["OPERAND1"], script).compare(
        &aux_value(shared, data, state, &block.inputs["OPERAND2"], script),
      ) < 0.,
    ),
    "operator_letter_of" => Value::String(aux_map_as_str(
      shared,
      data,
      state,
      &block.inputs["STRING"],
      script,
      |s| {
        s.chars()
          .nth(
            aux_f64(shared, data, state, &block.inputs["LETTER"], script) as usize - 1,
          )
          .and_then(|c| Some(c.to_string()))
          .unwrap_or(format!(""))
      },
    )),
    "operator_and" => Value::Bool(
      aux_bool(shared, data, state, &block.inputs["OPERAND1"], script)
        && aux_bool(shared, data, state, &block.inputs["OPERAND2"], script),
    ),
    "operator_or" => Value::Bool(
      aux_bool(shared, data, state, &block.inputs["OPERAND1"], script)
        || aux_bool(shared, data, state, &block.inputs["OPERAND2"], script),
    ),
    "operator_not" => Value::Bool(!aux_bool(
      shared,
      data,
      state,
      &block.inputs["OPERAND"],
      script,
    )),
    "operator_random" => Value::Float({
      let from = aux_value(shared, data, state, &block.inputs["FROM"], script);
      let to = aux_value(shared, data, state, &block.inputs["TO"], script);
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
      shared,
      data,
      state,
      &block.inputs["STRING1"],
      script,
      |s1| {
        aux_map_as_str(
          shared,
          data,
          state,
          &block.inputs["STRING2"],
          script,
          |s2| format!("{s1}{s2}"),
        )
      },
    )),
    "operator_length" => Value::Float(aux_map_as_str(
      shared,
      data,
      state,
      &block.inputs["STRING"],
      script,
      |s| s.len(),
    ) as f64),
    "operator_contains" => Value::Bool(aux_map_as_str(
      shared,
      data,
      state,
      &block.inputs["STRING1"],
      script,
      |s1| {
        aux_map_as_str(
          shared,
          data,
          state,
          &block.inputs["STRING2"],
          script,
          |s2| s1.to_lowercase().contains(s2.to_lowercase().as_str()),
        )
      },
    )),
    "operator_mod" => Value::Float({
      let n = aux_f64(shared, data, state, &block.inputs["NUM1"], script);
      let modulus = aux_f64(shared, data, state, &block.inputs["NUM2"], script);
      let mut result = n % modulus;
      if result / modulus < 0. {
        result += modulus;
      }
      result
    }),
    "operator_round" => {
      Value::Float(aux_f64(shared, data, state, &block.inputs["NUM"], script).round())
    }
    "operator_mathop" => {
      let value = aux_f64(shared, data, state, &block.inputs["NUM"], script);
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
    "data_lengthoflist" => {
      let Input::List(list) = &block.inputs["LIST"] else { panic!() };
      if list.is_global {
        panic!();
      }
      Value::Float(state.lists[list.id].len() as f64)
    }
    "data_itemoflist" => {
      let Input::List(list) = &block.inputs["LIST"] else { panic!() };
      if list.is_global {
        panic!();
      }
      let list = &state.lists[list.id];
      let index = aux_f64(shared, data, state, &block.inputs["INDEX"], script).floor();
      if 0. < index && index <= list.len() as f64 {
        list[index as usize - 1].clone()
      } else {
        Value::String(format!(""))
      }
    }
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

fn get_variable<'a>(
  shared: &'a SharedState,
  state: &'a TargetState,
  variable: &VariableInput,
) -> &'a Value {
  if variable.is_global {
    &shared.global_variables[variable.id]
  } else {
    &state.variables[variable.id]
  }
}

fn set_variable<'a, F: FnOnce(&Value) -> Value>(
  shared: &'a mut SharedState,
  state: &'a mut TargetState,
  variable: &'a VariableInput,
  map: F,
) {
  if variable.is_global {
    let value = map(&shared.global_variables[variable.id]);
    shared.global_variables[variable.id] = value;
  } else {
    let value = map(&state.variables[variable.id]);
    state.variables[variable.id] = value;
  }
}

fn aux_f64(
  shared: &SharedState,
  data: &TargetData,
  state: &TargetState,
  input: &Input,
  script: &Script,
) -> f64 {
  match input {
    Input::Block(id) => evaluate_block(shared, data, state, *id, script).to_f64(),
    Input::Value(value) => value.to_f64(),
    Input::Variable(variable) => get_variable(shared, state, variable).to_f64(),
    Input::Argument(argument) => get_argument(*argument, script).to_f64(),
    _ => 0.,
  }
}

fn aux_bool(
  shared: &SharedState,
  data: &TargetData,
  state: &TargetState,
  input: &Input,
  script: &Script,
) -> bool {
  match input {
    Input::Block(id) => evaluate_block(shared, data, state, *id, script).to_bool(),
    Input::Value(value) => value.to_bool(),
    Input::Variable(variable) => get_variable(shared, state, variable).to_bool(),
    Input::Argument(argument) => get_argument(*argument, script).to_bool(),
    _ => false,
  }
}

fn aux_string(
  shared: &SharedState,
  data: &TargetData,
  state: &TargetState,
  input: &Input,
  script: &Script,
) -> String {
  match input {
    Input::Block(id) => evaluate_block(shared, data, state, *id, script).to_string(),
    Input::Value(value) => value.to_string(),
    Input::Variable(variable) => get_variable(shared, state, variable).to_string(),
    Input::Argument(argument) => get_argument(*argument, script).to_string(),
    _ => panic!(),
  }
}

fn aux_value(
  shared: &SharedState,
  data: &TargetData,
  state: &TargetState,
  input: &Input,
  script: &Script,
) -> Value {
  match input {
    Input::Block(id) => evaluate_block(shared, data, state, *id, script),
    Input::Value(value) => value.clone(),
    Input::Variable(variable) => get_variable(shared, state, variable).clone(),
    Input::Argument(argument) => get_argument(*argument, script).clone(),
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
  shared: &SharedState,
  data: &TargetData,
  state: &TargetState,
  input: &Input,
  script: &Script,
  map: F,
) -> T {
  match input {
    Input::Block(id) => {
      evaluate_block(shared, data, state, *id, script).map_as_str(map)
    }
    Input::Value(value) => value.map_as_str(map),
    Input::Variable(variable) => get_variable(shared, state, variable).map_as_str(map),
    Input::Argument(argument) => get_argument(*argument, script).map_as_str(map),
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
  pub custom_blocks: HashMap<String, CustomBlock>,
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
  pub pen: PenState,
}

#[derive(Debug)]
pub struct PenState {
  pub is_down: bool,
  pub size: u32,
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub a: u8,

  pub x: f64,
  pub y: f64,
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
