mod audio;
mod background;
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
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    window.set_resizable(false);

    println!("Window size: {} {}", window.width(), window.height());

    let mut camera_bundle = OrthographicCameraBundle::new_2d();

    // the default makes it such that negative z is clipped
    // we need to use z: -1 for the background, so shift the camera a bit more forward
    camera_bundle.transform.translation.z = 500.0;

    commands.spawn_bundle(camera_bundle);
    commands.spawn_bundle(UiCameraBundle::default());
}
