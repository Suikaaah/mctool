#![windows_subsystem = "windows"]

mod coord;
mod engine;
mod grid;
mod io;
mod map_err_anyhow;
mod resources;
mod state;

fn detail() -> anyhow::Result<()> {
    use {engine::Engine, resources::Resources, state::State};

    let mut engine = Engine::new()?;
    let resources = Resources::new(engine.tex_creator())?;
    let mut state = State::new(&resources)?;
    let fonts = resources.load_fonts()?;

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

        state.step(&resources)?;
        engine.draw(&state, &fonts)?;
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
