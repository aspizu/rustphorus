mod block;
mod costume;
mod input;
mod script;
mod sprite;
mod stage;
mod state;
use crate::script::Script;
use crate::state::{load_state, State};
use sdl2::{
    event::Event,
    image::LoadTexture,
    pixels::Color,
    rect::Rect,
    render::{Canvas, TextureCreator},
    video::Window,
    video::WindowContext,
};
use std::time::Duration;

fn load_textures<'a>(state: &mut State<'a>, texture_creator: &'a TextureCreator<WindowContext>) {
    for costume in &state.stage.costumes {
        state
            .textures
            .entry(costume.md5ext.clone())
            .or_insert_with(|| texture_creator.load_texture(&costume.md5ext).unwrap());
    }
    for sprite in &state.sprites {
        for costume in &sprite.data.costumes {
            state
                .textures
                .entry(costume.md5ext.clone())
                .or_insert_with(|| texture_creator.load_texture(&costume.md5ext).unwrap());
        }
    }
}

fn render(state: &mut State, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();
    let costume = &state.stage.costumes[state.stage.current_costume];
    let texture = &state.textures[&costume.md5ext];
    let query = texture.query();
    let x: i32 = state.stage_width as i32 / 2 - query.width as i32 / 2;
    let y: i32 = state.stage_height as i32 / 2 - query.height as i32 / 2;
    canvas
        .copy(&texture, None, Rect::new(x, y, query.width, query.height))
        .unwrap();
    for sprite in &state.sprites {
        let costume = &sprite.data.costumes[sprite.state.current_costume];
        let texture = &state.textures[&costume.md5ext];
        let query = texture.query();
        let scale = sprite.state.size / costume.bitmap_resolution;
        let width = query.width * scale / 100;
        let height = query.height * scale / 100;
        let x: i32 = state.stage_width as i32 / 2 + sprite.state.x - width as i32 / 2;
        let y: i32 = state.stage_height as i32 / 2 + sprite.state.y - height as i32 / 2;
        canvas
            .copy(&texture, None, Rect::new(x, y, width, height))
            .unwrap();
    }
}

fn start_scripts(state: &mut State) {
    for sprite in &mut state.sprites {
        for (id, block) in &sprite.data.blocks {
            match block.opcode.as_str() {
                "event_whenflagclicked" => {
                    sprite.scripts.push(Script { id: id.clone() });
                }
                _ => {}
            }
        }
    }
}

fn step_all_scripts(state: &mut State) {
    for sprite in &mut state.sprites {
        sprite.step_scripts();
    }
}

fn main() {
    let state: State = load_state();
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Rustphorus", state.stage_width, state.stage_height)
        .opengl()
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut state = state;
    load_textures(&mut state, &texture_creator);
    start_scripts(&mut state);
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'main;
                }
                _ => {}
            }
        }
        step_all_scripts(&mut state);
        render(&mut state, &mut canvas);
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / state.frame_rate));
    }
}
