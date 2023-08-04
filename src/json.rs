use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use sdl2::image::LoadTexture;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;
use serde::{Deserialize, Deserializer};

use crate::block;
use crate::block::Value;
use crate::project;
use crate::project::Config;
use crate::project::Texture;
use crate::target;
use serde::de::SeqAccess;
use serde::de::Visitor;
use std::fmt;
use std::fmt::Formatter;

#[derive(Deserialize)]
pub struct Project {
  targets: Vec<Target>,
  // extensions: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Target {
  is_stage: bool,
  name: String,
  variables: HashMap<String, Variable>,
  lists: HashMap<String, List>,
  blocks: HashMap<String, Block>,
  current_costume: usize,
  costumes: Vec<Costume>,
  // layer_order: i32,
  volume: f64,
  // tempo: f64,
  #[serde(default = "default_true")]
  visible: bool,
  #[serde(default = "default_f64")]
  x: f64,
  #[serde(default = "default_f64")]
  y: f64,
  #[serde(default = "default_size")]
  size: f64,
  #[serde(default = "default_direction")]
  direction: f64,
  #[serde(default = "default_false")]
  draggable: bool,
  #[serde(default = "default_rotation_style")]
  rotation_style: String,
}

fn default_true() -> bool {
  true
}

fn default_false() -> bool {
  false
}

fn default_f64() -> f64 {
  0.
}

fn default_size() -> f64 {
  100.
}

fn default_direction() -> f64 {
  90.
}

fn default_rotation_style() -> String {
  format!("don't rotate")
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Block {
  opcode: String,
  next: Option<String>,
  parent: Option<String>,
  inputs: HashMap<String, Input>,
  // fields: HashMap<String, Field>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Costume {
  name: String,
  md5ext: String,
  // data_format: String,
  bitmap_resolution: u32,
  rotation_center_x: f64,
  rotation_center_y: f64,
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

#[derive(Debug)]
pub enum Input {
  Block(String),
  Value(Value),
  Broadcast(BroadcastInput),
  Variable(VariableInput),
  List(ListInput),
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
      fn visit_seq<A: SeqAccess<'de>>(
        self,
        mut seq: A,
      ) -> Result<Self::Value, A::Error> {
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
          Some(T::Values(mut values)) => match values[0].to_i32() {
            4 | 5 | 6 | 7 | 8 | 9 | 10 => {
              return Ok(Input::Value(values.remove(1)));
            }
            11 => {
              return Ok(Input::Broadcast(BroadcastInput {
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
            12 => {
              return Ok(Input::Variable(VariableInput {
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
            13 => {
              return Ok(Input::List(ListInput {
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
            _ => panic!(),
          },
          None => panic!(),
        }
      }
    }
    de.deserialize_seq(SeqVisitor)
  }
}

#[derive(Debug)]
pub struct BroadcastInput {
  pub name: String,
  pub id: String,
}

#[derive(Debug)]
pub struct VariableInput {
  pub name: String,
  pub id: String,
}

#[derive(Debug)]
pub struct ListInput {
  pub name: String,
  pub id: String,
}

pub fn load<'a>(
  texture_creator: &'a TextureCreator<WindowContext>,
  config: Config,
) -> project::Project<'a> {
  let json_project: Project =
    serde_json::from_reader(BufReader::new(File::open("tmp/project.json").unwrap()))
      .unwrap();
  let mut project = project::Project {
    config,
    target_name_to_target_index: HashMap::with_capacity(json_project.targets.len()), // DONE
    targets: Vec::with_capacity(json_project.targets.len()), // DONE
    textures: Vec::new(),                                    // DONE
  };
  let json_stage = &json_project.targets[0];
  let mut global_variables_id_to_index: HashMap<&String, usize> =
    HashMap::with_capacity(json_stage.variables.len());
  let mut global_lists_id_to_index: HashMap<&String, usize> =
    HashMap::with_capacity(json_stage.lists.len());
  let mut index = 0;
  for id in json_stage.variables.keys() {
    global_variables_id_to_index.insert(&id, index);
    index += 1;
  }
  let mut index = 0;
  for id in json_project.targets[0].lists.keys() {
    global_lists_id_to_index.insert(&id, index);
    index += 1;
  }
  let mut costume_md5ext_to_texture_index: HashMap<&String, usize> = HashMap::new();
  for json_target in &json_project.targets {
    for costume in &json_target.costumes {
      let md5ext = &costume.md5ext;
      costume_md5ext_to_texture_index
        .entry(md5ext)
        .or_insert_with(|| {
          project.textures.push(Texture {
            bitmap_resolution: costume.bitmap_resolution,
            texture: texture_creator
              .load_texture(format!("tmp/{md5ext}"))
              .unwrap(),
            rotation_center_x: costume.rotation_center_x,
            rotation_center_y: costume.rotation_center_y,
          });
          project.textures.len() - 1
        });
    }
    project
      .target_name_to_target_index
      .insert(json_target.name.clone(), project.targets.len());
    project.targets.push(target::Target {
      data: target::TargetData {
        is_stage: json_target.is_stage,
        blocks: Vec::new(),
        costume_name_to_index: HashMap::new(), // DONE
        costume_index_to_texture_index: HashMap::new(), // DONE
      },
      state: target::TargetState {
        visible: json_target.visible,
        x: json_target.x,
        y: json_target.y,
        size: json_target.size,
        direction: json_target.direction,
        draggable: json_target.draggable,
        current_costume: json_target.current_costume,
        rotation_style: match json_target.rotation_style.as_str() {
          "all around" => target::RotationStyle::AllAround,
          "don't rotate" => target::RotationStyle::DontRotate,
          "left-right" => target::RotationStyle::LeftRight,
          _ => panic!(),
        },
        volume: json_target.volume,
        variables: Vec::with_capacity(json_target.variables.len()), // DONE
        lists: Vec::with_capacity(json_target.lists.len()),         // DONE
      },
      scripts: Vec::new(),
    });
    let target = project.targets.last_mut().unwrap();

    let mut variables_id_to_index: HashMap<&String, usize> =
      HashMap::with_capacity(json_target.variables.len());
    let mut lists_id_to_index: HashMap<&String, usize> =
      HashMap::with_capacity(json_target.lists.len());
    if json_target.is_stage {
      for (id, index) in &global_variables_id_to_index {
        if *index >= target.state.variables.len() {
          target
            .state
            .variables
            .resize(*index + 1, block::Value::Float(0.));
        }
        target.state.variables[*index] = json_target.variables[*id].value.clone();
      }
      for (id, index) in &global_lists_id_to_index {
        if *index >= target.state.lists.len() {
          target.state.lists.resize(*index + 1, vec![]);
        }
        target.state.lists[*index] = json_target.lists[*id].value.clone();
      }
    } else {
      for (id, variable) in &json_target.variables {
        variables_id_to_index.insert(&id, target.state.variables.len());
        target.state.variables.push(variable.value.clone());
      }
      for (id, list) in &json_target.lists {
        lists_id_to_index.insert(&id, target.state.lists.len());
        target.state.lists.push(list.value.clone());
      }
    }
    let mut id_to_index: HashMap<&String, usize> =
      HashMap::with_capacity(json_target.blocks.len());
    let mut index_to_id: Vec<&String> = Vec::with_capacity(id_to_index.len());
    let mut index: usize = 1;
    for id in json_target.blocks.keys() {
      id_to_index.insert(id, index);
      index_to_id.push(id);
      index += 1;
    }
    for id in index_to_id {
      let block = &json_target.blocks[id];
      target.data.blocks.push(block::Block {
        opcode: block.opcode.clone(),
        next: match &block.next {
          Some(next) => id_to_index[&next],
          None => 0,
        },
        parent: match &block.parent {
          Some(parent) => id_to_index[&parent],
          None => 0,
        },
        inputs: block
          .inputs
          .iter()
          .map(|(key, input)| {
            (
              key.clone(),
              match &input {
                Input::Value(value) => block::Input::Value(value.clone()),
                Input::Block(id) => block::Input::Block(id_to_index[&id]),
                Input::Broadcast(broadcast) => {
                  block::Input::Broadcast(block::BroadcastInput {
                    name: broadcast.name.clone(),
                    id: broadcast.id.clone(),
                  })
                }
                Input::Variable(variable) => block::Input::Variable(
                  variables_id_to_index
                    .get(&variable.id)
                    .and_then(|id| {
                      Some(block::VariableInput {
                        is_global: false,
                        id: *id,
                      })
                    })
                    .unwrap_or_else(|| block::VariableInput {
                      is_global: true,
                      id: global_variables_id_to_index[&variable.id],
                    }),
                ),
                Input::List(list) => block::Input::List(block::ListInput {
                  is_global: false,
                  id: lists_id_to_index[&list.id],
                }),
              },
            )
          })
          .collect(),
      });
    }
    for (i, costume) in json_target.costumes.iter().enumerate() {
      target
        .data
        .costume_name_to_index
        .insert(costume.name.clone(), i);
      target
        .data
        .costume_index_to_texture_index
        .insert(i, costume_md5ext_to_texture_index[&costume.md5ext]);
    }
  }
  project
}
