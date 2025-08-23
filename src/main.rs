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

    'main_loop: loop {
        while let Some(event) = engine.poll_event() {
            if let sdl2::event::Event::Quit { .. } = event {
                break 'main_loop;
            }
        }

        state.step()?;
        engine.draw(&state, &font)?;
        Engine::sleep();
    }

    Ok(())
}

fn main() {
    if let Err(e) = detail() {
        io::message_box(format!("Reason: {e}"), "Program terminated")
            .expect("failed to show message box");
    }
}
