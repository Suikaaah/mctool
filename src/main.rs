#![windows_subsystem = "windows"]

mod coord;
mod engine;
mod font_engine;
mod grid;
mod io;
mod map_err_anyhow;
mod state;

fn detail() -> anyhow::Result<()> {
    use {engine::Engine, font_engine::FontEngine, state::State};

    const FONT: &str = "CascadiaMono.ttf";
    const FONT_SIZE: u16 = 16;

    let mut engine = Engine::new()?;
    let mut state = State::new()?;
    let font_engine = FontEngine::new()?;
    let font = font_engine.load_font(FONT, FONT_SIZE)?;

    engine.start_text_input();

    'main_loop: loop {
        while let Some(event) = engine.poll_event() {
            use sdl2::{event::Event, keyboard::Keycode};

            match event {
                Event::Quit { .. } => break 'main_loop,
                Event::TextInput { text, .. } => state.push_text(&text),
                Event::KeyDown {
                    keycode: Some(Keycode::BACKSPACE),
                    ..
                } => state.pop_text(),
                _ => (),
            }
        }

        state.step()?;
        engine.draw(&state, &font)?;
        Engine::sleep();
    }

    engine.stop_text_input();

    Ok(())
}

fn main() {
    if let Err(e) = detail() {
        io::message_box(format!("Reason: {e}"), "Program terminated")
            .expect("failed to show message box");
    }
}
