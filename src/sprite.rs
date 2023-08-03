use serde::Deserialize;
use std::collections::HashMap;

use crate::{
    block::{Block, List, Value, Variable},
    costume::Costume,
    input::Input,
    script::{Branch, RepeatBranch, Script},
};

#[derive(Debug, Deserialize)]
pub struct Sprite<'a> {
    #[serde(flatten)]
    pub data: SpriteData,
    #[serde(flatten)]
    pub state: SpriteState,
    #[serde(skip_deserializing)]
    pub scripts: Vec<Script<'a>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteState {
    pub layer_order: usize,
    pub visible: bool,
    pub x: f64,
    pub y: f64,
    pub size: f64,
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

// Thanks to the guys at the Rust Discord Server.
impl<'b> Sprite<'b> {
    fn evaluate_block(data: &SpriteData, _state: &SpriteState, id: &String) -> Value {
        let _block = &data.blocks[id];
        Value::Float(0.)
    }

    fn aux_i32(data: &SpriteData, state: &SpriteState, input: &Input) -> i32 {
        match input {
            Input::Block(id) => Sprite::evaluate_block(data, state, id).to_i32(),
            Input::Value(value) => value.to_i32(),
            Input::Variable(variable) => state.variables[&variable.id].value.to_i32(),
            _ => 0,
        }
    }

    fn aux_f64(data: &SpriteData, state: &SpriteState, input: &Input) -> f64 {
        match input {
            Input::Block(id) => Sprite::evaluate_block(data, state, id).to_f64(),
            Input::Value(value) => value.to_f64(),
            Input::Variable(variable) => state.variables[&variable.id].value.to_f64(),
            _ => 0.,
        }
    }

    fn aux_id<'a>(input: &'a Input) -> &'a String {
        match input {
            Input::Block(id) => id,
            _ => panic!(),
        }
    }

    fn execute_block<'a>(
        script: &'a mut Script<'a>,
        data: &'a SpriteData,
        state: &mut SpriteState,
    ) -> Option<&'a str> {
        let mut id = &script.id;
        let mut pop: bool = false;

        if !data.blocks[*id].next.is_some() {
            if let Some(branch) = script.stack.last_mut() {
                match branch {
                    Branch::Repeat(branch) => {
                        println!("Sprite({}) inside Repeat({})", data.name, branch.iterations);
                        branch.iterations -= 1;
                        if branch.iterations <= 0 {
                            pop = true;
                            id = &&branch.return_id;
                        } else {
                            return data.blocks[branch.branch_id]
                                .next
                                .as_ref()
                                .map(|s| s.as_str());
                        }
                    }
                    _ => panic!(),
                }
            }
        }

        let block = &data.blocks[*id];
        println!("Sprite({}) :: {}", data.name, block.opcode);

        match block.opcode.as_str() {
            "event_whenflagclicked" => {}
            "motion_gotoxy" => {
                state.x = Sprite::aux_f64(data, state, &block.inputs["X"]);
                state.y = Sprite::aux_f64(data, state, &block.inputs["Y"]);
            }
            "motion_changexby" => {
                state.x += Sprite::aux_f64(data, state, &block.inputs["DX"]);
            }
            "motion_changeyby" => {
                state.y += Sprite::aux_f64(data, state, &block.inputs["DY"]);
            }
            "control_repeat" => {
                let branch_id = Sprite::aux_id(&block.inputs["SUBSTACK"]);
                script.stack.push(Branch::Repeat(RepeatBranch {
                    iterations: Sprite::aux_i32(data, state, &block.inputs["TIMES"]) as u32,
                    return_id: id,
                    branch_id: branch_id,
                }));
                return Some(branch_id);
            }
            _ => panic!(),
        }

        if pop {
            script.stack.pop();
        }

        block.next.as_ref().map(|s| s.as_str())
    }

    pub fn step_scripts(&mut self) {
        self.scripts
            .retain_mut(|script| Sprite::step_script(&self.data, &mut self.state, script))
    }

    fn step_script<'a>(
        data: &'a SpriteData,
        state: &'a mut SpriteState,
        script: &'a mut Script<'a>,
    ) -> bool {
        if let Some(next) = Sprite::execute_block(script, data, state) {
            script.id = next;
            true
        } else {
            println!("terminated");
            false
        }
    }
}
