//! Second Story entry point: window setup, the capture harness and the frame loop.

use macroquad::prelude::*;
use macroquad_toolkit::capture;

use apartment::game::Game;
use apartment::{save, ui};

fn window_conf() -> Conf {
    capture::capture_window_conf("APARTMENT", "Second Story", 1280, 720)
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;

    // Screenshot harness: when APARTMENT_CAPTURE_PATH is set, seed a scene,
    // simulate deterministic frames, write a PNG, and exit.
    if let Some(configs) = capture::CaptureConfig::all_from_env("APARTMENT") {
        for config in configs {
            game.begin_capture_scene(&config.scene);
            capture::run_capture_once(&config, |_dt| {
                clear_background(ui::theme::color::BACKGROUND());
                game.update();
                game.draw();
            })
            .await;
        }
        return;
    }

    let settings = save::load_settings();
    settings.apply_effects();
    if settings.fullscreen {
        set_fullscreen(true);
    }

    loop {
        clear_background(ui::theme::color::BACKGROUND());
        game.update();
        game.draw();
        next_frame().await;
    }
}
