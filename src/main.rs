mod audio;
mod game_state;
mod ingame_ui;
mod loading;
mod mover;
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
use player::{Player, PlayerKilledEvent, PlayerPlugin};
use score::ScorePlugin;

// TODO: Remove ALL these if possible
use loading::LoadingAssets;
use player::PlayerCrossedPillarEvent;

use crate::mover::{Mover, MoverWindowLeftDespawnBound};

const PILLAR_GAP: f32 = 150.0;
const PILLAR_HEIGHT: f32 = 1024.0;
const PILLAR_WIDTH: f32 = 128.0;
const PLAYER_VISIBLE_HEIGHT: f32 = 46.0;

const NEXT_PILLAR_SPAWN_TIME: f32 = 3.0;

const PILLAR_SPEED: f32 = 150.0;

#[derive(Component)]
struct GameOverText;

#[derive(Component)]
struct Pillar {
    player_crossed: bool,
}

struct PillarPool(Vec<Entity>);

struct PillarSpawnerTimer(Timer);

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
        .add_startup_system(setup)
        .insert_resource(PillarSpawnerTimer(Timer::from_seconds(
            NEXT_PILLAR_SPAWN_TIME,
            true,
        )))
        .insert_resource(PillarPool(vec![]))
        .add_system_set(
            SystemSet::new()
                .label("logic")
                .before("events")
                .with_system(player_pillar_check_system)
                .with_system(pillar_spawn_system)
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
    mut pillar_pools: ResMut<PillarPool>,
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
    let pillar_top = asset_server.load("pillar_top.png");
    let pillar_bottom = asset_server.load("pillar_bottom.png");

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

    pillar_pools.0.extend((0..10).map(|_| {
        commands
            .spawn()
            .insert(Pillar {
                player_crossed: false,
            })
            .insert(Transform {
                translation: Vec3::new(window.width(), 0.0, 0.0),
                ..Default::default()
            })
            .insert(GlobalTransform {
                ..Default::default()
            })
            .insert(Mover {
                active: false,
                velocity: Vec3::new(-PILLAR_SPEED, 0.0, 0.0),
                acceleration: Vec3::ZERO,
            })
            .insert(MoverWindowLeftDespawnBound {
                object_width: PILLAR_WIDTH,
            })
            .with_children(|parent| {
                parent.spawn_bundle(SpriteBundle {
                    texture: pillar_top.clone(),
                    transform: Transform {
                        translation: Vec3::new(
                            0.0,
                            (PILLAR_HEIGHT / 2.0) + (PILLAR_GAP / 2.0),
                            0.0,
                        ),
                        ..Default::default()
                    },
                    ..Default::default()
                });

                parent.spawn_bundle(SpriteBundle {
                    texture: pillar_bottom.clone(),
                    transform: Transform {
                        translation: Vec3::new(
                            0.0,
                            -(PILLAR_HEIGHT / 2.0) - (PILLAR_GAP / 2.0),
                            0.0,
                        ),
                        ..Default::default()
                    },
                    ..Default::default()
                });
            })
            .id()
    }));

    loading.0.push(background.clone_untyped());
    loading.0.push(pillar_top.clone_untyped());
    loading.0.push(pillar_bottom.clone_untyped());
}

fn player_pillar_check_system(
    game_status: Res<GameState>,
    mut query: Query<(&Transform, &mut Pillar, &Mover), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
    mut cross_event: EventWriter<PlayerCrossedPillarEvent>,
    mut killed_event: EventWriter<PlayerKilledEvent>,
) {
    let player_transform = player_query.single();

    if game_state::is_playing(&game_status) {
        query.iter_mut().for_each(|(transform, mut pillar, mover)| {
            if mover.active
                && transform.translation.x <= (PILLAR_WIDTH / 2.0)
                && transform.translation.x >= -(PILLAR_WIDTH / 2.0)
            {
                let top = PILLAR_GAP / 2.0 + transform.translation.y;
                let bottom = -PILLAR_GAP / 2.0 + transform.translation.y;

                if player_transform.translation.y > top - (PLAYER_VISIBLE_HEIGHT / 2.0)
                    || player_transform.translation.y < bottom + (PLAYER_VISIBLE_HEIGHT / 2.0)
                {
                    killed_event.send(PlayerKilledEvent);
                // divide by 4.0 => to allow player to score when he reaches 75% across the pillar
                } else if transform.translation.x < -(PILLAR_WIDTH / 4.0) && !pillar.player_crossed
                {
                    pillar.player_crossed = true;
                    cross_event.send(PlayerCrossedPillarEvent);
                }
            }
        });
    }
}

fn pillar_spawn_system(
    windows: Res<Windows>,
    time: Res<Time>,
    game_status: Res<GameState>,
    mut timer: ResMut<PillarSpawnerTimer>,
    pillar_pools: Res<PillarPool>,
    mut pillar_query: Query<(&mut Pillar, &mut Transform, &mut Mover)>,
) {
    if game_state::is_playing(&game_status) && timer.0.tick(time.delta()).just_finished() {
        let window = windows.get_primary().unwrap();
        let window_width = window.width() as f32;
        let window_height = window.height() as f32;

        let mut found = false;

        for child in pillar_pools.0.iter() {
            let (mut pillar, mut transform, mut mover) = pillar_query.get_mut(*child).unwrap();
            if !mover.active {
                let gap_y = ((rand::random::<f32>() - 0.5) * 2.0) * ((window_height - 100.0) / 2.0);

                mover.active = true;
                pillar.player_crossed = false;
                transform.translation.x = (window_width / 2.0) + (PILLAR_WIDTH / 2.0);
                transform.translation.y = gap_y;

                found = true;
                break;
            }
        }

        if !found {
            eprintln!("Exhausted pillars in pool");
        }
    }
}

fn restart_system(
    game_status: Res<GameState>,
    windows: Res<Windows>,
    keyboard_input: Res<Input<KeyCode>>,
    mut pillar_query: Query<(&mut Mover, &mut Pillar, &mut Transform), Without<Player>>,
    mut timer: ResMut<PillarSpawnerTimer>,
    mut start_new_events: EventWriter<StartNewGameEvent>,
) {
    if game_state::is_game_over(&game_status) && keyboard_input.just_pressed(KeyCode::R) {
        let window = windows.get_primary().unwrap();
        let window_width = window.width();

        start_new_events.send(StartNewGameEvent);

        pillar_query
            .iter_mut()
            .for_each(|(mut mover, mut pillar, mut transform)| {
                mover.active = false;
                pillar.player_crossed = false;

                // hack to avoid dealing with visibility
                // (have to modify children which is troublesome...)
                transform.translation.x = window_width;
            });

        timer.0.reset();
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
