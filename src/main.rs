use sdl2::{
    event::Event,
    image::LoadTexture,
    pixels::Color,
    rect::Rect,
    render::{Canvas, TextureCreator},
    video::Window,
    video::WindowContext,
};
use state::{load_virtual_machine_state, State};

pub mod state;

extern crate sdl2;

fn load_textures(state: &State, texture_creator: TextureCreator<WindowContext>) {
    for costume in &state.stage.costumes {
        state
            .textures
            .entry(costume.md5ext)
            .or_insert_with(|| texture_creator.load_texture(costume.md5ext).unwrap());
    }
}

fn render(state: &State, canvas: &Canvas<Window>) {
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
        let costume = &sprite.costumes[sprite.current_costume];
        let texture = &state.textures[&costume.md5ext];
        let query = texture.query();
        let scale = sprite.size / costume.bitmap_resolution;
        let width = query.width * scale / 100;
        let height = query.height * scale / 100;
        let x: i32 = state.stage_width as i32 / 2 + sprite.x - width as i32 / 2;
        let y: i32 = state.stage_height as i32 / 2 + sprite.y - height as i32 / 2;
        canvas
            .copy(&texture, None, Rect::new(x, y, width, height))
            .unwrap();
    }
}

fn main() {
    let state: State = load_virtual_machine_state();
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Rustphorus", state.stage_width, state.stage_height)
        .position_centered()
        .build()
        .unwrap();
    let canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    load_textures(&state, texture_creator);
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'main;
                }
                _ => {}
            }
        }
        render(&state, &canvas);
    }
}
