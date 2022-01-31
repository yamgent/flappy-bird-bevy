use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioPlugin, AudioSource};

const PILLAR_GAP: f32 = 150.0;
const PILLAR_HEIGHT: f32 = 1024.0;
const PILLAR_WIDTH: f32 = 128.0;
const PLAYER_VISIBLE_HEIGHT: f32 = 46.0;

const NEXT_PILLAR_SPAWN_TIME: f32 = 3.0;

const GRAVITY: f32 = 9.81;
const LEAP_Y_VELOCITY: f32 = 5.0;

#[derive(Component)]
struct Player {
    y_velocity: f32,
}

#[derive(Component)]
struct GameOverText;

#[derive(Component)]
struct Pillar {
    active: bool,
    player_crossed: bool,
}

struct PillarPool(Vec<Entity>);

struct PillarSpawnerTimer(Timer);

#[derive(Clone, Copy)] // TODO: Remove this when event is no longer using GameState?
enum GameState {
    Loading,
    StartScreen,
    Playing,
    GameOver,
}

struct Globals {
    game_state: GameState,
    score: u32,
}

#[derive(Component)]
struct ScoreText;

struct AudioCollection {
    crossed: Handle<AudioSource>,
    dead: Handle<AudioSource>,
}

#[derive(Component)]
struct StartScreenText;

struct LoadingAssets(Vec<HandleUntyped>);

struct ChangeGameStateEvent(GameState);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_startup_system(setup)
        .insert_resource(LoadingAssets(vec![]))
        .insert_resource(PillarSpawnerTimer(Timer::from_seconds(
            NEXT_PILLAR_SPAWN_TIME,
            true,
        )))
        .insert_resource(Globals {
            game_state: GameState::Loading,
            score: 0,
        })
        .insert_resource(PillarPool(vec![]))
        .add_event::<ChangeGameStateEvent>()
        .add_system_set(
            SystemSet::new()
                .label("logic")
                .with_system(player_gravity_system)
                .with_system(pillar_movement_system)
                .with_system(pillar_spawn_system)
                .with_system(restart_system)
                .with_system(main_ui_system)
                .with_system(pregame_ui_system),
        )
        .add_system_set(
            SystemSet::new()
                .label("events")
                .after("logic")
                .with_system(game_state_event_system),
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
    let crossed = asset_server.load("crossed.wav");
    let dead = asset_server.load("dead.wav");

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
        .insert(Player { y_velocity: 0.0 });

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

    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: "Score: ".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 30.0,
                            color: Color::BLACK,
                        },
                    },
                    TextSection {
                        value: "0".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 30.0,
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
                    top: Val::Px(15.0),
                    left: Val::Px(15.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(ScoreText);

    pillar_pools.0.extend((0..10).map(|_| {
        commands
            .spawn()
            .insert(Pillar {
                active: false,
                player_crossed: false,
            })
            .insert(Transform {
                translation: Vec3::new(window.width(), 0.0, 0.0),
                ..Default::default()
            })
            .insert(GlobalTransform {
                ..Default::default()
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

    commands.insert_resource(AudioCollection {
        crossed: crossed.clone(),
        dead: dead.clone(),
    });

    loading.0.push(background.clone_untyped());
    loading.0.push(player.clone_untyped());
    loading.0.push(font.clone_untyped());
    loading.0.push(pillar_top.clone_untyped());
    loading.0.push(pillar_bottom.clone_untyped());
    loading.0.push(crossed.clone_untyped());
    loading.0.push(dead.clone_untyped());
}

fn player_gravity_system(
    windows: Res<Windows>,
    time: Res<Time>,
    globals: Res<Globals>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Player, &mut Transform)>,
    audio: Res<Audio>,
    audio_collection: Res<AudioCollection>,
    mut change_game_state_event: EventWriter<ChangeGameStateEvent>,
) {
    let (mut player, mut transform) = query.single_mut();

    if matches!(globals.game_state, GameState::Playing) {
        if keyboard_input.just_pressed(KeyCode::Space) {
            player.y_velocity = LEAP_Y_VELOCITY;
        } else {
            player.y_velocity -= time.delta().as_secs_f32() * GRAVITY;
        }

        transform.translation.y += player.y_velocity;

        let window = windows.get_primary().unwrap();

        let (min_y, max_y) = (-window.height() as f32 / 2.0, window.height() as f32 / 2.0);

        if transform.translation.y < min_y || transform.translation.y > max_y {
            change_game_state_event.send(ChangeGameStateEvent(GameState::GameOver));
            audio.play(audio_collection.dead.clone());
        }
    }
}

fn pillar_movement_system(
    windows: Res<Windows>,
    time: Res<Time>,
    mut globals: ResMut<Globals>,
    mut query: Query<(&mut Transform, &mut Pillar), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
    audio: Res<Audio>,
    audio_collection: Res<AudioCollection>,
    mut change_game_state_event: EventWriter<ChangeGameStateEvent>,
) {
    let window = windows.get_primary().unwrap();
    let window_width = window.width() as f32;

    let player_transform = player_query.single();

    if matches!(globals.game_state, GameState::Playing) {
        query.iter_mut().for_each(|(mut transform, mut pillar)| {
            if pillar.active {
                transform.translation.x -= time.delta().as_secs_f32() * 150.0;

                if transform.translation.x <= (PILLAR_WIDTH / 2.0)
                    && transform.translation.x >= -(PILLAR_WIDTH / 2.0)
                {
                    let top = PILLAR_GAP / 2.0 + transform.translation.y;
                    let bottom = -PILLAR_GAP / 2.0 + transform.translation.y;

                    if player_transform.translation.y > top - (PLAYER_VISIBLE_HEIGHT / 2.0)
                        || player_transform.translation.y < bottom + (PLAYER_VISIBLE_HEIGHT / 2.0)
                    {
                        change_game_state_event.send(ChangeGameStateEvent(GameState::GameOver));

                        audio.play(audio_collection.dead.clone());
                    // divide by 4.0 => to allow player to score when he reaches 75% across the pillar
                    } else if transform.translation.x < -(PILLAR_WIDTH / 4.0)
                        && !pillar.player_crossed
                    {
                        pillar.player_crossed = true;
                        audio.play(audio_collection.crossed.clone());
                        globals.score += 1;
                    }
                } else if transform.translation.x < (-window_width / 2.0) - PILLAR_WIDTH {
                    pillar.active = false;
                    pillar.player_crossed = false;
                }
            }

            if !pillar.active {
                // move it out of the viewport
                transform.translation.x = window_width;
            }
        });
    }
}

fn pillar_spawn_system(
    windows: Res<Windows>,
    time: Res<Time>,
    globals: Res<Globals>,
    mut timer: ResMut<PillarSpawnerTimer>,
    pillar_pools: Res<PillarPool>,
    mut pillar_query: Query<(&mut Pillar, &mut Transform)>,
) {
    if matches!(globals.game_state, GameState::Playing)
        && timer.0.tick(time.delta()).just_finished()
    {
        let window = windows.get_primary().unwrap();
        let window_width = window.width() as f32;
        let window_height = window.height() as f32;

        let mut found = false;

        for child in pillar_pools.0.iter() {
            let (mut pillar, mut transform) = pillar_query.get_mut(*child).unwrap();
            if !pillar.active {
                let gap_y = ((rand::random::<f32>() - 0.5) * 2.0) * ((window_height - 100.0) / 2.0);

                pillar.active = true;
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
    mut globals: ResMut<Globals>,
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
    mut pillar_query: Query<&mut Pillar>,
    mut timer: ResMut<PillarSpawnerTimer>,
    mut change_game_state_event: EventWriter<ChangeGameStateEvent>,
) {
    if matches!(globals.game_state, GameState::GameOver) && keyboard_input.just_pressed(KeyCode::R)
    {
        change_game_state_event.send(ChangeGameStateEvent(GameState::Playing));
        globals.score = 0;

        let (mut player, mut player_transform) = player_query.single_mut();

        player_transform.translation = Vec3::ZERO;
        player.y_velocity = 0.0;

        pillar_query.iter_mut().for_each(|mut pillar| {
            pillar.active = false;
            pillar.player_crossed = false;
        });

        timer.0.reset();
    }
}

fn main_ui_system(globals: Res<Globals>, mut query: Query<&mut Text, With<ScoreText>>) {
    let mut text = query.single_mut();

    text.sections[1].value = globals.score.to_string();
}

fn pregame_ui_system(
    globals: Res<Globals>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Text, &mut Visibility), With<StartScreenText>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    loading: Option<Res<LoadingAssets>>,
    mut change_game_state_event: EventWriter<ChangeGameStateEvent>,
) {
    match globals.game_state {
        GameState::Loading => {
            use bevy::asset::LoadState;

            match asset_server.get_group_load_state(loading.unwrap().0.iter().map(|h| h.id)) {
                LoadState::Failed => {
                    query.iter_mut().for_each(|(mut text, _)| {
                        text.sections[0].value = "Loading failed...".to_string();
                    });
                }
                LoadState::Loaded => {
                    change_game_state_event.send(ChangeGameStateEvent(GameState::StartScreen));

                    query.iter_mut().for_each(|(mut text, _)| {
                        text.sections[0].value = "Press <Space> to Start".to_string();
                    });

                    commands.remove_resource::<LoadingAssets>();
                }
                _ => {}
            }
        }
        GameState::StartScreen => {
            if keyboard_input.just_pressed(KeyCode::Space) {
                change_game_state_event.send(ChangeGameStateEvent(GameState::Playing));

                query.iter_mut().for_each(|(_, mut visibility)| {
                    visibility.is_visible = false;
                });
            }
        }
        _ => {}
    }
}

fn game_state_event_system(
    mut globals: ResMut<Globals>,
    mut events: EventReader<ChangeGameStateEvent>,
    mut game_over_query: Query<&mut Visibility, With<GameOverText>>,
) {
    events.iter().for_each(|event| {
        globals.game_state = event.0;

        let mut game_over_visibility = game_over_query.single_mut();
        game_over_visibility.is_visible = matches!(event.0, GameState::GameOver);
    });
}
