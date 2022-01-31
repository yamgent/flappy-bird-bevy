mod audio;
mod game_state;
mod ingame_ui;
mod loading;
mod player;
mod score;

use audio::GameAudioPlugin;
use bevy::prelude::*;
use game_state::{
    GameState, GameStatePlugin, GameStateType, OnGameStateChangedEvent, StartNewGameEvent,
};
use ingame_ui::IngameUiPlugin;
use loading::{FinishLoadingEvent, LoadingManagerPlugin};
use player::PlayerPlugin;
use score::ScorePlugin;

// TODO: Remove ALL these if possible
use loading::LoadingAssets;
use player::PlayerCrossedPillarEvent;
use score::ResetScoreEvent;

const PILLAR_GAP: f32 = 150.0;
const PILLAR_HEIGHT: f32 = 1024.0;
const PILLAR_WIDTH: f32 = 128.0;
const PLAYER_VISIBLE_HEIGHT: f32 = 46.0;

const NEXT_PILLAR_SPAWN_TIME: f32 = 3.0;

const PLAYER_GRAVITY: f32 = 9.81 * 60.0;
const LEAP_Y_VELOCITY: f32 = 5.0 * 60.0;
const PILLAR_SPEED: f32 = 150.0;

#[derive(Component)]
struct Mover {
    active: bool,
    velocity: Vec3,
    acceleration: Vec3,
}

#[derive(Component)]
struct MoverWindowLeftDespawnBound {
    object_width: f32,
}

#[derive(Component)]
struct Player;

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

struct PlayerKilledEvent;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ScorePlugin)
        .add_plugin(GameStatePlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(GameAudioPlugin)
        .add_plugin(LoadingManagerPlugin)
        .add_plugin(IngameUiPlugin)
        .add_startup_system(setup)
        .insert_resource(PillarSpawnerTimer(Timer::from_seconds(
            NEXT_PILLAR_SPAWN_TIME,
            true,
        )))
        .insert_resource(PillarPool(vec![]))
        .add_event::<PlayerKilledEvent>()
        .add_system_set(
            SystemSet::new()
                .label("physics")
                .before("input")
                .with_system(mover_system)
                .with_system(mover_window_left_despawn_bound_system),
        )
        .add_system_set(
            SystemSet::new()
                .label("input")
                .before("logic")
                .with_system(player_input_system),
        )
        .add_system_set(
            SystemSet::new()
                .label("logic")
                .before("events")
                .with_system(player_bounds_check_system)
                .with_system(player_pillar_check_system)
                .with_system(pillar_spawn_system)
                .with_system(restart_system)
                .with_system(pregame_ui_system),
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
    let player = asset_server.load("player.png");
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
        .spawn_bundle(SpriteBundle {
            texture: player.clone(),
            ..Default::default()
        })
        .insert(Player)
        .insert(Mover {
            active: true,
            velocity: Vec3::ZERO,
            acceleration: Vec3::new(0.0, -PLAYER_GRAVITY, 0.0),
        });

    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "Loading...".to_string(),
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
    loading.0.push(player.clone_untyped());
    loading.0.push(font.clone_untyped());
    loading.0.push(pillar_top.clone_untyped());
    loading.0.push(pillar_bottom.clone_untyped());
}

fn mover_system(
    game_status: Res<GameState>,
    time: Res<Time>,
    mut query: Query<(&mut Mover, &mut Transform)>,
) {
    // TODO: Is the coupling with game_state reasonable?
    if game_state::is_playing(&game_status) {
        query.iter_mut().for_each(|(mut mover, mut transform)| {
            if mover.active {
                let increment = mover.acceleration * time.delta().as_secs_f32();
                mover.velocity += increment;
                transform.translation += mover.velocity * time.delta().as_secs_f32();
            }
        });
    }
}

fn mover_window_left_despawn_bound_system(
    game_status: Res<GameState>,
    windows: Res<Windows>,
    mut query: Query<(&MoverWindowLeftDespawnBound, &mut Mover, &mut Transform)>,
) {
    let window = windows.get_primary().unwrap();
    let window_width = window.width() as f32;

    if game_state::is_playing(&game_status) {
        query
            .iter_mut()
            .for_each(|(mover_window_bound, mut mover, mut transform)| {
                if mover.active
                    && transform.translation.x
                        < (-window_width / 2.0) - (mover_window_bound.object_width / 2.0)
                {
                    mover.active = false;

                    // hack to avoid dealing with visibility
                    // (have to modify children which is troublesome...)
                    transform.translation.x = window_width;
                }
            });
    }
}

fn player_input_system(
    game_status: Res<GameState>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Mover, With<Player>>,
) {
    let mut mover = query.single_mut();

    if game_state::is_playing(&game_status) && keyboard_input.just_pressed(KeyCode::Space) {
        mover.velocity.y = LEAP_Y_VELOCITY;
    }
}

fn player_bounds_check_system(
    windows: Res<Windows>,
    game_status: Res<GameState>,
    mut query: Query<&Transform, With<Player>>,
    mut killed_event: EventWriter<PlayerKilledEvent>,
) {
    let transform = query.single_mut();

    if game_state::is_playing(&game_status) {
        let window = windows.get_primary().unwrap();

        let (min_y, max_y) = (-window.height() as f32 / 2.0, window.height() as f32 / 2.0);

        if transform.translation.y < min_y || transform.translation.y > max_y {
            killed_event.send(PlayerKilledEvent);
        }
    }
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
    mut player_query: Query<(&mut Mover, &mut Transform), With<Player>>,
    mut pillar_query: Query<(&mut Mover, &mut Pillar, &mut Transform), Without<Player>>,
    mut timer: ResMut<PillarSpawnerTimer>,
    mut reset_score_events: EventWriter<ResetScoreEvent>,
    mut start_new_events: EventWriter<StartNewGameEvent>,
) {
    if game_state::is_game_over(&game_status) && keyboard_input.just_pressed(KeyCode::R) {
        let window = windows.get_primary().unwrap();
        let window_width = window.width();

        start_new_events.send(StartNewGameEvent);
        reset_score_events.send(ResetScoreEvent);

        let (mut mover, mut player_transform) = player_query.single_mut();

        player_transform.translation = Vec3::ZERO;
        mover.velocity = Vec3::ZERO;

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

fn pregame_ui_system(
    game_status: Res<GameState>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Text, &mut Visibility), With<StartScreenText>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    loading: Option<Res<LoadingAssets>>,
    mut start_new_events: EventWriter<StartNewGameEvent>,
    mut finish_loading_events: EventWriter<FinishLoadingEvent>,
) {
    match game_status.0 {
        GameStateType::Loading => {
            use bevy::asset::LoadState;

            match asset_server.get_group_load_state(loading.unwrap().0.iter().map(|h| h.id)) {
                LoadState::Failed => {
                    query.iter_mut().for_each(|(mut text, _)| {
                        text.sections[0].value = "Loading failed...".to_string();
                    });
                }
                LoadState::Loaded => {
                    finish_loading_events.send(FinishLoadingEvent);

                    query.iter_mut().for_each(|(mut text, _)| {
                        text.sections[0].value = "Press <Space> to Start".to_string();
                    });

                    commands.remove_resource::<LoadingAssets>();
                }
                _ => {}
            }
        }
        GameStateType::StartScreen => {
            if keyboard_input.just_pressed(KeyCode::Space) {
                start_new_events.send(StartNewGameEvent);

                query.iter_mut().for_each(|(_, mut visibility)| {
                    visibility.is_visible = false;
                });
            }
        }
        _ => {}
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
