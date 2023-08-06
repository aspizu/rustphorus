use crate::block::Value;

#[derive(Debug)]
pub struct Script {
  pub id: usize,
  pub stack: Vec<StackFrame>,
  pub arguments: Vec<Value>,
  pub arguments_start: usize,
  pub refresh: bool,
}

#[derive(Debug)]
pub enum StackFrame {
  Repeat {
    iterations: u32,
    jump_id: usize,
    return_id: usize,
  },
  Goto(usize),
  CustomBlock {
    argument_count: usize,
    return_id: usize,
    refresh_was_set_false: bool,
  },
}
