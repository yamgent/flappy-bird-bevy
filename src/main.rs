mod audio;
mod game_state;
mod ingame_ui;
mod loading;
mod mover;
mod pillars;
mod player;
mod score;

use audio::GameAudioPlugin;
use bevy::prelude::*;
use game_state::{
    GameState, GameStatePlugin, GameStateType, OnGameStateChangedEvent, StartNewGameEvent,
};
use ingame_ui::IngameUiPlugin;
use loading::LoadingManagerPlugin;
use mover::MoverPlugin;
use pillars::PillarsPlugin;
use player::{PlayerKilledEvent, PlayerPlugin};
use score::ScorePlugin;

// TODO: Remove ALL these if possible
use loading::LoadingAssets;

#[derive(Component)]
struct GameOverText;

#[derive(Component)]
struct StartScreenText;

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
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .label("logic")
                .before("events")
                .with_system(restart_system)
                .with_system(start_screen_ui_system),
        )
        .add_system_set(
            SystemSet::new()
                .label("events")
                .with_system(game_over_ui_update_system),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut windows: ResMut<Windows>,
    mut loading: ResMut<LoadingAssets>,
) {
    let window = windows.get_primary_mut().unwrap();
    window.set_resizable(false);

    println!("Window size: {} {}", window.width(), window.height());

    let mut camera_bundle = OrthographicCameraBundle::new_2d();

    // the default makes it such that negative z is clipped
    // we need to use z: -1 for the background, so shift the camera a bit more forward
    camera_bundle.transform.translation.z = 500.0;

    commands.spawn_bundle(camera_bundle);
    commands.spawn_bundle(UiCameraBundle::default());

    let background = asset_server.load("background.png");
    let font = asset_server.load("FiraSans-Bold.ttf");

    commands.spawn_bundle(SpriteBundle {
        texture: background.clone(),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, -1.0),
            ..Default::default()
        },
        ..Default::default()
    });

    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        font: font.clone(),
                        font_size: 60.0,
                        color: Color::BLACK,
                    },
                }],
                alignment: TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                },
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Percent(50.0),
                    left: Val::Percent(50.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(StartScreenText);

    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: "Game Over!".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 60.0,
                            color: Color::RED,
                        },
                    },
                    TextSection {
                        value: "  Press <R> to restart".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 40.0,
                            color: Color::BLACK,
                        },
                    },
                ],
                alignment: TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                },
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Percent(50.0),
                    left: Val::Percent(50.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(GameOverText);

    loading.0.push(background.clone_untyped());
}

fn restart_system(
    game_status: Res<GameState>,
    keyboard_input: Res<Input<KeyCode>>,
    mut start_new_events: EventWriter<StartNewGameEvent>,
) {
    if game_state::is_game_over(&game_status) && keyboard_input.just_pressed(KeyCode::R) {
        start_new_events.send(StartNewGameEvent);
    }
}

fn start_screen_ui_system(
    game_status: Res<GameState>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Text, &mut Visibility), With<StartScreenText>>,
    mut start_new_events: EventWriter<StartNewGameEvent>,
) {
    if let GameStateType::StartScreen = game_status.0 {
        // TODO: Don't continuously set this
        query.iter_mut().for_each(|(mut text, _)| {
            text.sections[0].value = "Press <Space> to start".to_string()
        });

        if keyboard_input.just_pressed(KeyCode::Space) {
            start_new_events.send(StartNewGameEvent);

            query.iter_mut().for_each(|(_, mut visibility)| {
                visibility.is_visible = false;
            });
        }
    }
}

fn game_over_ui_update_system(
    mut game_status_changed: EventReader<OnGameStateChangedEvent>,
    mut game_over_query: Query<&mut Visibility, With<GameOverText>>,
) {
    game_status_changed.iter().for_each(|event| {
        let mut game_over_visibility = game_over_query.single_mut();
        game_over_visibility.is_visible = matches!(event.0, GameStateType::GameOver);
    });
}
