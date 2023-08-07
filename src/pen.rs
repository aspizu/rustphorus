use std::collections::LinkedList;

use sdl2::{pixels::Color, render::Canvas, video::Window};

#[derive(Debug)]
pub struct PenInstruction {
  pub size: u32,
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub a: u8,
  pub x1: f64,
  pub y1: f64,
  pub x2: f64,
  pub y2: f64,
}

pub fn render_pen(
  stage_width: u32,
  stage_height: u32,
  canvas: &mut Canvas<Window>,
  instructions: &LinkedList<PenInstruction>,
) {
  for line in instructions {
    canvas.set_draw_color(Color {
      r: line.r,
      g: line.g,
      b: line.b,
      a: line.a,
    });
    canvas
      .draw_line(
        (
          (line.x1 + stage_width as f64 / 2.) as i32,
          (stage_height as f64 / 2. - line.y1) as i32,
        ),
        (
          (line.x2 + stage_width as f64 / 2.) as i32,
          (stage_height as f64 / 2. - line.y2) as i32,
        ),
      )
      .unwrap();
  }
}
