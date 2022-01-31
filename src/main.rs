mod audio;
mod background;
mod game_core;
mod game_state;
mod ingame_ui;
mod loading;
mod mover;
mod pillars;
mod player;
mod score;
mod screen_end;
mod screen_start;

use audio::GameAudioPlugin;
use background::BackgroundPlugin;
use bevy::prelude::*;
use game_core::GameCorePlugin;
use game_state::GameStatePlugin;
use ingame_ui::IngameUiPlugin;
use loading::LoadingManagerPlugin;
use mover::MoverPlugin;
use pillars::PillarsPlugin;
use player::PlayerPlugin;
use score::ScorePlugin;
use screen_end::ScreenEndPlugin;
use screen_start::ScreenStartPlugin;

#[derive(Component)]
struct GameOverText;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(GameCorePlugin)
        .add_plugin(ScorePlugin)
        .add_plugin(GameStatePlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(GameAudioPlugin)
        .add_plugin(LoadingManagerPlugin)
        .add_plugin(IngameUiPlugin)
        .add_plugin(MoverPlugin)
        .add_plugin(PillarsPlugin)
        .add_plugin(ScreenStartPlugin)
        .add_plugin(ScreenEndPlugin)
        .add_plugin(BackgroundPlugin)
        .run();
}
