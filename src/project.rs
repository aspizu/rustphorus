use std::{collections::HashMap, fs::remove_dir_all, process::Command};

use derivative::Derivative;
use sdl2::{
  pixels::Color,
  render::Canvas,
  render::{Texture as sdl2Texture, TextureCreator},
  video::Window,
  video::WindowContext,
};

use crate::{json, target::Target};

#[derive(Debug)]
pub struct Project<'a> {
  pub config: Config,
  pub target_name_to_target_index: HashMap<String, usize>,
  pub targets: Vec<Target>,
  pub textures: Vec<Texture<'a>>,
}

#[derive(Debug)]
pub struct Config {
  pub stage_width: u32,
  pub stage_height: u32,
  pub frame_rate: u32,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Texture<'a> {
  pub bitmap_resolution: u32,
  #[derivative(Debug = "ignore")]
  pub texture: sdl2Texture<'a>,
  pub rotation_center_x: f64,
  pub rotation_center_y: f64,
}

impl<'a> Project<'a> {
  pub fn load(
    path: &str,
    texture_creator: &'a TextureCreator<WindowContext>,
    config: Config,
  ) -> Project<'a> {
    Command::new("unzip")
      .arg("-o")
      .arg(path)
      .arg("-d")
      .arg("tmp")
      .status()
      .unwrap();
    json::load(&texture_creator, config)
  }
  pub fn render(&self, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::WHITE);
    canvas.clear();
    for target in &self.targets {
      target.render(&self.textures, canvas, &self.config);
    }
  }

  pub fn start_scripts(&mut self) {
    for target in &mut self.targets {
      target.start_scripts();
    }
  }

  pub fn execute_scripts(&mut self) {
    for target in &mut self.targets {
      target.execute_scripts();
    }
  }
}

impl<'a> Drop for Project<'a> {
  fn drop(&mut self) {
    remove_dir_all("tmp").unwrap();
  }
}
