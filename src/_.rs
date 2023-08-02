extern crate sdl2;

use sb3::Details;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;

fn main() {
    Command::new("unzip").arg("project.sb3").output().unwrap();

    let project = sb3::load("project.json");

    let stage_width: i32 = 480;
    let stage_height: i32 = 360;
    let frame_rate: u32 = 30;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Rustphorus", stage_width as u32, stage_height as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let texture_creator = canvas.texture_creator();

    let mut costumes = HashMap::new();
    for costume in project.targets.iter().flat_map(|target| &target.costumes) {
        costumes
            .entry(costume.md5ext.clone())
            .or_insert_with(|| texture_creator.load_texture(&costume.md5ext).unwrap());
    }

    /* let dango = texture_creator.load_texture("dango.png").unwrap(); */

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'main;
                }
                _ => {}
            }
        }
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();
        for target in &project.targets {
            let costume = &target.costumes[target.current_costume];
            let costume_texture = &costumes[&costume.md5ext];
            let query = costume_texture.query();
            match &target.details {
                Details::Sprite(sprite) => {
                    let scale = sprite.size / (costume.bitmap_resolution as u32);
                    let width = query.width * scale / 100;
                    let height = query.height * scale / 100;
                    let x: i32 = (stage_width / 2) + sprite.x - (width as i32 / 2);
                    let y: i32 = (stage_height / 2) + sprite.y - (height as i32 / 2);
                    canvas
                        .copy(&costume_texture, None, Rect::new(x, y, width, height))
                        .unwrap();
                }
                Details::Stage(_) => {
                    let x: i32 = (stage_width / 2) - (query.width as i32 / 2);
                    let y: i32 = (stage_height / 2) - (query.height as i32 / 2);
                    canvas
                        .copy(
                            &costume_texture,
                            None,
                            Rect::new(x, y, query.width, query.height),
                        )
                        .unwrap();
                }
            }
        }
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / frame_rate));
    }
}
