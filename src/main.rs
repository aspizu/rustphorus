use project::{Config, Project};
use sdl2::event::Event;
use std::{thread::sleep, time::Duration};

mod block;
mod json;
mod project;
mod script;
mod target;

fn main() {
  pretty_env_logger::init();
  let config = Config {
    stage_width: 480,
    stage_height: 360,
    frame_rate: 30,
  };
  let sdl_context = sdl2::init().unwrap();
  let ttf_context = sdl2::ttf::init().unwrap();
  let font = ttf_context.load_font("font.ttf", 16).unwrap();
  let video_subsystem = sdl_context.video().unwrap();
  let window = video_subsystem
    .window("Rustphorus", config.stage_width, config.stage_height)
    .opengl()
    .position_centered()
    .build()
    .unwrap();
  let mut canvas = window.into_canvas().build().unwrap();
  let mut event_pump = sdl_context.event_pump().unwrap();
  let texture_creator = canvas.texture_creator();
  let mut project = Project::load("project.sb3", &texture_creator, config);
  //println!("{project:#?}");
  //panic!();
  project.start_scripts();
  let duration = Duration::new(0, 1_000_000_000u32 / project.config.frame_rate);
  'main: loop {
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. } => {
          break 'main;
        }
        _ => {}
      }
    }
    project.render(&mut canvas, &texture_creator, &font);
    project.execute_scripts();
    canvas.present();
    sleep(duration);
  }
}
