use bevy::prelude::*;

const PILLAR_GAP: f32 = 150.0;
const PILLAR_HEIGHT: f32 = 1024.0;
const PLAYER_VISIBLE_HEIGHT: f32 = 46.0;

#[derive(Component)]
struct Player {
    y_velocity: f32,
}

#[derive(Component)]
struct GameOverText;

#[derive(Component)]
struct Pillar {
    active: bool,
}

#[derive(Component)]
struct PillarPool;

struct PillarSpawnerTimer(Timer);

enum GameState {
    Playing,
    GameOver,
}

struct Globals {
    game_state: GameState,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .insert_resource(PillarSpawnerTimer(Timer::from_seconds(3.0, true)))
        .insert_resource(Globals {
            game_state: GameState::Playing,
        })
        .add_system(player_gravity_system)
        .add_system(game_over_ui_text_system)
        .add_system(pillar_movement_system)
        .add_system(pillar_spawn_system)
        .add_system(restart_system)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("player.png"),
            ..Default::default()
        })
        .insert(Player { y_velocity: 0.0 });

    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: "Game Over!".to_string(),
                        style: TextStyle {
                            font: asset_server.load("FiraSans-Bold.ttf"),
                            font_size: 60.0,
                            color: Color::RED,
                        },
                    },
                    TextSection {
                        value: "  Press <R> to restart".to_string(),
                        style: TextStyle {
                            font: asset_server.load("FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::WHITE,
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
        .spawn()
        .insert(PillarPool)
        .insert(Transform {
            ..Default::default()
        })
        .insert(GlobalTransform {
            ..Default::default()
        })
        .with_children(|parent| {
            (0..10).for_each(|_| {
                parent
                    .spawn()
                    .insert(Pillar { active: false })
                    .insert(Transform {
                        ..Default::default()
                    })
                    .insert(GlobalTransform {
                        ..Default::default()
                    })
                    .with_children(|gparent| {
                        gparent.spawn_bundle(SpriteBundle {
                            texture: asset_server.load("pillar_top.png"),
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

                        gparent.spawn_bundle(SpriteBundle {
                            texture: asset_server.load("pillar_bottom.png"),
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
                    });
            });
        });
}

fn player_gravity_system(
    windows: Res<Windows>,
    time: Res<Time>,
    mut globals: ResMut<Globals>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    let (mut player, mut transform) = query.single_mut();

    if matches!(globals.game_state, GameState::Playing) {
        if keyboard_input.just_pressed(KeyCode::Space) {
            player.y_velocity = 5.0;
        } else {
            player.y_velocity -= time.delta().as_secs_f32() * 9.81;
        }

        transform.translation.y += player.y_velocity;

        let window = windows.get_primary().unwrap();

        let (min_y, max_y) = (-window.height() as f32 / 2.0, window.height() as f32 / 2.0);

        if transform.translation.y < min_y || transform.translation.y > max_y {
            globals.game_state = GameState::GameOver;
        }
    }
}

fn game_over_ui_text_system(
    globals: Res<Globals>,
    mut query: Query<(&mut Visibility, &GameOverText)>,
) {
    let (mut visibility, _) = query.single_mut();

    visibility.is_visible = matches!(globals.game_state, GameState::GameOver);
}

fn pillar_movement_system(
    windows: Res<Windows>,
    time: Res<Time>,
    mut globals: ResMut<Globals>,
    mut query: Query<(&mut Transform, &mut Pillar), Without<Player>>,
    player_query: Query<(&Player, &Transform)>,
) {
    let window = windows.get_primary().unwrap();
    let window_width = window.width() as f32;

    let (_, player_transform) = player_query.single();

    if matches!(globals.game_state, GameState::Playing) {
        query.iter_mut().for_each(|(mut transform, mut pillar)| {
            if pillar.active {
                transform.translation.x -= time.delta().as_secs_f32() * 150.0;

                if transform.translation.x <= 64.0 && transform.translation.x >= -64.0 {
                    let top = PILLAR_GAP / 2.0 + transform.translation.y;
                    let bottom = -PILLAR_GAP / 2.0 + transform.translation.y;

                    if player_transform.translation.y > top - (PLAYER_VISIBLE_HEIGHT / 2.0)
                        || player_transform.translation.y < bottom + (PLAYER_VISIBLE_HEIGHT / 2.0)
                    {
                        globals.game_state = GameState::GameOver;
                    }
                } else if transform.translation.x < (-window_width / 2.0) - 200.0 {
                    pillar.active = false;
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
    query: Query<(&PillarPool, &Children)>,
    mut children_query: Query<(&mut Pillar, &mut Transform)>,
) {
    if matches!(globals.game_state, GameState::Playing)
        && timer.0.tick(time.delta()).just_finished()
    {
        let window = windows.get_primary().unwrap();
        let window_width = window.width() as f32;
        let window_height = window.height() as f32;

        let (_, children) = query.single();
        let mut found = false;

        for &child in children.iter() {
            let (mut pillar, mut transform) = children_query.get_mut(child).unwrap();
            if !pillar.active {
                let gap_y = ((rand::random::<f32>() - 0.5) * 2.0) * ((window_height - 100.0) / 2.0);

                pillar.active = true;
                transform.translation.x = window_width / 2.0;
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
) {
    if matches!(globals.game_state, GameState::GameOver) && keyboard_input.just_pressed(KeyCode::R)
    {
        globals.game_state = GameState::Playing;

        let (mut player, mut player_transform) = player_query.single_mut();

        player_transform.translation = Vec3::ZERO;
        player.y_velocity = 0.0;

        pillar_query.iter_mut().for_each(|mut pillar| {
            pillar.active = false;
        });

        timer.0.reset();
    }
}
