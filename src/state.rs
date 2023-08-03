use crate::sprite::Sprite;
use crate::stage::Stage;
use sdl2::render::Texture;
use serde::{de::Visitor, Deserializer};
use serde::{
    de::{Error, SeqAccess},
    Deserialize,
};
use std::fmt;
use std::process::Command;
use std::{collections::HashMap, fs::File, io::BufReader};

pub struct State<'a> {
    pub stage: Stage,
    pub sprites: Vec<Sprite>,
    pub stage_width: u32,
    pub stage_height: u32,
    pub frame_rate: u32,
    pub textures: HashMap<String, Texture<'a>>,
}

pub fn load_state() -> State<'static> {
    Command::new("unzip")
        .arg("-o")
        .arg("Project.sb3")
        .status()
        .unwrap();
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

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
