#![allow(
    clippy::enum_variant_names,
    clippy::large_enum_variant,
    clippy::module_inception,
    clippy::too_many_arguments
)]

use macroquad::prelude::*;
use macroquad_toolkit::capture;

mod building;
mod data;
mod economy;
mod game;
mod simulation;
mod state;
mod tenant;
mod ui;

mod assets;
mod save;

// Headless balance-simulation harness (test-only).
#[cfg(test)]
mod sim_harness;

// Phase 3 modules
mod city;
mod consequences;
mod narrative;
mod util;

use game::Game;

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

    loop {
        clear_background(ui::theme::color::BACKGROUND());
        game.update();
        game.draw();
        next_frame().await;
    }
}
