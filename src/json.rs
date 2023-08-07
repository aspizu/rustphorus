use std::collections::{HashMap, LinkedList};
use std::fs::File;
use std::io::BufReader;

use sdl2::image::LoadTexture;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};

use crate::block;
use crate::block::CustomBlock;
use crate::block::Value;
use crate::project::Config;
use crate::project::Texture;
use crate::project::{self, SharedState};
use crate::target::{self, PenState};
use serde::de::SeqAccess;
use serde::de::Visitor;
use std::fmt;
use std::fmt::Formatter;

#[derive(Deserialize)]
pub struct Project {
  targets: Vec<Target>,
  // extensions: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Target {
  is_stage: bool,
  name: String,
  variables: HashMap<String, Variable>,
  lists: HashMap<String, List>,
  blocks: HashMap<String, Block>,
  current_costume: i32,
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
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Block {
  opcode: String,
  next: Option<String>,
  parent: Option<String>,
  inputs: HashMap<String, Input>,
  fields: HashMap<String, Field>,
  #[serde(default = "no_mutation")]
  mutation: Mutation,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Mutation {
  #[serde(default)]
  proccode: String,
  #[serde(default, deserialize_with = "parse_json")]
  argumentids: Vec<String>,
  #[serde(default, deserialize_with = "parse_json")]
  argumentnames: Vec<String>,
  #[serde(default, deserialize_with = "parse_json")]
  warp: bool,
}

fn no_mutation() -> Mutation {
  Mutation {
    proccode: format!(""),
    argumentids: vec![],
    argumentnames: vec![],
    warp: false,
  }
}

fn parse_json<'de, D, T>(de: D) -> Result<T, D::Error>
where
  D: Deserializer<'de>,
  T: DeserializeOwned,
{
  let json_string = <String>::deserialize(de).unwrap();
  let array: T = serde_json::from_str(json_string.as_str()).unwrap();
  Ok(array)
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug)]
pub struct Field {
  pub value: Value,
  pub id: Option<String>,
}

impl<'de> Deserialize<'de> for Field {
  fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
    struct SeqVisitor;

    impl<'de> Visitor<'de> for SeqVisitor {
      type Value = Field;

      fn expecting(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Field")
      }

      fn visit_seq<A: SeqAccess<'de>>(
        self,
        mut seq: A,
      ) -> Result<Self::Value, A::Error> {
        match seq.next_element::<Value>()? {
          Some(value) => match seq.next_element::<String>() {
            Ok(Some(id)) => Ok(Field {
              value,
              id: Some(id),
            }),
            _ => Ok(Field { value, id: None }),
          },
          _ => panic!(),
        }
      }
    }

    de.deserialize_seq(SeqVisitor)
  }
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
          Values(Option<Vec<Value>>),
        }
        let _shadow = seq.next_element::<i32>()?.unwrap();
        match seq.next_element::<T>()? {
          Some(T::String(string)) => {
            while seq.next_element::<serde_json::Value>()?.is_some() {}
            return Ok(Input::Block(string));
          }
          Some(T::Values(values)) => {
            //
            if values.is_none() {
              while seq.next_element::<serde_json::Value>()?.is_some() {}
              return Ok(Input::Value(Value::Float(0.)));
            }
            let mut values = values.unwrap();
            match values[0].to_f64() as i32 {
              4 | 5 | 6 | 7 | 8 | 9 | 10 => {
                while seq.next_element::<serde_json::Value>()?.is_some() {}
                return Ok(Input::Value(values.remove(1)));
              }
              11 => {
                while seq.next_element::<serde_json::Value>()?.is_some() {}
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
                while seq.next_element::<serde_json::Value>()?.is_some() {}
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
                while seq.next_element::<serde_json::Value>()?.is_some() {}
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
            }
          }
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

fn convert_argument_reporter(
  blocks: &HashMap<String, Block>,
  block: &Block,
) -> Option<usize> {
  // reporter block is rogue, no need to convert.
  let Some(mut id) = block.parent.as_ref() else { return None };

  let custom_block_id;
  loop {
    let block = &blocks[id];
    if block.opcode == "procedures_definition" {
      if let Input::Block(id) = &block.inputs["custom_block"] {
        custom_block_id = id;
        break;
      } else {
        panic!("procedures_definition must have a custom_block input.");
      }
    }
    if let Some(parent_id) = &block.parent {
      id = parent_id;
    } else {
      // reporter block is contained inside a rogue stack of blocks, return random argument id.
      // This id should never be used!
      return Some(0);
    }
  }
  let custom_block = &blocks[custom_block_id];

  let Value::String(argument_name) = &block.fields["VALUE"].value else { return None };

  let index = custom_block
    .mutation
    .argumentnames
    .iter()
    .position(|x| x == argument_name)
    .unwrap();

  Some(index)
}

pub fn load<'a>(
  texture_creator: &'a TextureCreator<WindowContext>,
  config: Config,
) -> project::Project<'a> {
  let mut json_project: Project =
    serde_json::from_reader(BufReader::new(File::open("tmp/project.json").unwrap()))
      .unwrap();

  // Convert fields into inputs
  for target in &mut json_project.targets {
    // Convert argument reporters
    let apply: Vec<(String, usize)> = target
      .blocks
      .iter()
      .filter_map(|(id, block)| {
        if block.opcode == "argument_reporter_string_number" {
          if let Some(index) = convert_argument_reporter(&target.blocks, block) {
            Some((id.clone(), index))
          } else {
            None
          }
        } else {
          None
        }
      })
      .collect();
    for (id, index) in apply {
      target
        .blocks
        .get_mut(&id)
        .unwrap()
        .inputs
        .insert(format!("VALUE"), Input::Value(Value::Float(index as f64)));
    }
    for block in target.blocks.values_mut() {
      // Convert mutation into inputs
      if block.opcode == "procedures_call" {
        block.inputs.insert(
          format!("PROCCODE"),
          Input::Value(Value::String(block.mutation.proccode.clone())),
        );
      }

      for (key, field) in &block.fields {
        if block.opcode == "argument_reporter_string_number" {
          continue;
        }
        block.inputs.insert(
          key.clone(),
          if let Some(id) = &field.id {
            if key == "VARIABLE" {
              Input::Variable(VariableInput {
                name: format!("ghost"),
                id: id.clone(),
              })
            } else if key == "LIST" {
              Input::List(ListInput {
                name: format!("ghost"),
                id: id.clone(),
              })
            } else if key == "BROADCAST_OPTION" {
              Input::Broadcast(BroadcastInput {
                name: field.value.to_string(),
                id: id.clone(),
              })
            } else {
              panic!()
            }
          } else {
            Input::Value(field.value.clone())
          },
        );
      }
    }
  }

  let mut project = project::Project {
    config,
    target_name_to_target_index: HashMap::with_capacity(json_project.targets.len()), // DONE
    targets: Vec::with_capacity(json_project.targets.len()), // DONE
    textures: Vec::new(),                                    // DONE
    shared_state: SharedState {
      global_variables: Vec::new(),
      global_lists: Vec::new(),
      pen: LinkedList::new(),
    },
  };
  let json_stage = &json_project.targets[0];
  let mut global_variables_id_to_index: HashMap<&String, usize> =
    HashMap::with_capacity(json_stage.variables.len());
  let mut global_lists_id_to_index: HashMap<&String, usize> =
    HashMap::with_capacity(json_stage.lists.len());
  let mut index = 0;
  for (id, variable) in &json_stage.variables {
    project
      .shared_state
      .global_variables
      .push(variable.value.clone());
    global_variables_id_to_index.insert(&id, index);
    index += 1;
  }
  let mut index = 0;
  for (id, list) in &json_stage.lists {
    project.shared_state.global_lists.push(list.value.clone());
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
        costume_index_to_name: Vec::with_capacity(json_target.costumes.len()), // DONE
        costume_name_to_index: HashMap::with_capacity(json_target.costumes.len()), // DONE
        costume_index_to_texture_index: HashMap::with_capacity(
          json_target.costumes.len(),
        ), // DONE
        custom_blocks: HashMap::new(),
      },
      state: target::TargetState {
        pen: PenState {
          is_down: false,
          size: 1,
          r: 0,
          g: 0,
          b: 255,
          a: 0,
          x: json_target.x,
          y: json_target.y,
        },
        visible: json_target.visible,
        x: json_target.x,
        y: json_target.y,
        size: json_target.size,
        direction: json_target.direction,
        draggable: json_target.draggable,
        current_costume: (json_target.current_costume) as usize,
        rotation_style: match json_target.rotation_style.as_str() {
          "all around" => target::RotationStyle::AllAround,
          "don't rotate" => target::RotationStyle::DontRotate,
          "left-right" => target::RotationStyle::LeftRight,
          _ => panic!(),
        },
        volume: json_target.volume,
        variables: Vec::with_capacity(json_target.variables.len()), // DONE
        lists: Vec::with_capacity(json_target.lists.len()),         // DONE
        say: None,
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
      // Filter custom blocks
      if block.opcode == "procedures_definition" {
        if let Input::Block(id) = &block.inputs["custom_block"] {
          let custom_block = &json_target.blocks[id];
          target.data.custom_blocks.insert(
            custom_block.mutation.proccode.clone(),
            CustomBlock {
              next: block
                .next
                .as_ref()
                .and_then(|next| Some(id_to_index[next]))
                .unwrap_or(0),
              argument_ids: custom_block.mutation.argumentids.clone(),
              refresh: !custom_block.mutation.warp,
            },
          );
        }
      }
      target.data.blocks.push(block::Block {
        opcode: block.opcode.clone(),
        next: match &block.next {
          Some(next) => id_to_index[&next],
          None => 0,
        },
        // parent: match &block.parent {
        //   Some(parent) => id_to_index[&parent],
        //   None => 0,
        // },
        inputs: block
          .inputs
          .iter()
          .map(|(key, input)| {
            (
              key.clone(),
              match input {
                Input::Value(value) => block::Input::Value(value.clone()),
                Input::Block(id) => {
                  let input_block = &json_target.blocks[id];
                  if input_block.opcode == "argument_reporter_string_number" {
                    if let Input::Value(value) = &input_block.inputs["VALUE"] {
                      if let Value::Float(index) = value {
                        block::Input::Argument(*index as usize)
                      } else {
                        panic!("{input_block:#?}")
                      }
                    } else {
                      panic!()
                    }
                  } else {
                    block::Input::Block(id_to_index[&id])
                  }
                }
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
                Input::List(list) => block::Input::List(
                  lists_id_to_index
                    .get(&list.id)
                    .and_then(|id| {
                      Some(block::ListInput {
                        is_global: false,
                        id: *id,
                      })
                    })
                    .unwrap_or_else(|| block::ListInput {
                      is_global: true,
                      id: global_lists_id_to_index[&list.id],
                    }),
                ),
              },
            )
          })
          .collect(),
      });
    }
    for (i, costume) in json_target.costumes.iter().enumerate() {
      target.data.costume_index_to_name.push(costume.name.clone());
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
