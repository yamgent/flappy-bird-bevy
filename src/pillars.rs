use bevy::prelude::*;

use crate::{
    game_state::{GameState, StartNewGameEvent},
    loading::LoadingAssets,
    mover::{Mover, MoverWindowLeftDespawnBound},
    player::{Player, PlayerCrossedPillarEvent, PlayerKilledEvent},
};

pub struct PillarsPlugin;

impl Plugin for PillarsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PillarSpawnerTimer(Timer::from_seconds(
            NEXT_PILLAR_SPAWN_TIME,
            true,
        )))
        .insert_resource(PillarPool(vec![]))
        .add_startup_system(setup_pillars)
        .add_system(new_game_system)
        .add_system(player_pillar_check_system)
        .add_system(pillar_spawn_system);
    }
}

const PILLAR_GAP: f32 = 150.0;
const PILLAR_HEIGHT: f32 = 1024.0;
const PILLAR_WIDTH: f32 = 128.0;
const PLAYER_VISIBLE_HEIGHT: f32 = 46.0;

const NEXT_PILLAR_SPAWN_TIME: f32 = 3.0;

const PILLAR_SPEED: f32 = 150.0;

struct PillarPool(Vec<Entity>);
struct PillarSpawnerTimer(Timer);

#[derive(Component)]
struct Pillar {
    player_crossed: bool,
}

fn setup_pillars(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut windows: ResMut<Windows>,
    mut loading: ResMut<LoadingAssets>,
    mut pillar_pools: ResMut<PillarPool>,
) {
    let window = windows.get_primary_mut().unwrap();

    let pillar_top = asset_server.load("pillar_top.png");
    let pillar_bottom = asset_server.load("pillar_bottom.png");

    loading.0.push(pillar_top.clone_untyped());
    loading.0.push(pillar_bottom.clone_untyped());

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
}

fn new_game_system(
    windows: Res<Windows>,
    mut start_new_events: EventReader<StartNewGameEvent>,
    mut query: Query<(&mut Mover, &mut Pillar, &mut Transform)>,
    mut timer: ResMut<PillarSpawnerTimer>,
) {
    if start_new_events.iter().count() > 0 {
        let window = windows.get_primary().unwrap();
        let window_width = window.width();

        query
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

fn player_pillar_check_system(
    game_state: Res<GameState>,
    mut query: Query<(&Transform, &mut Pillar, &Mover), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
    mut cross_event: EventWriter<PlayerCrossedPillarEvent>,
    mut killed_event: EventWriter<PlayerKilledEvent>,
) {
    let player_transform = player_query.single();

    if crate::game_state::is_playing(&game_state) {
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
    game_state: Res<GameState>,
    mut timer: ResMut<PillarSpawnerTimer>,
    pillar_pools: Res<PillarPool>,
    mut pillar_query: Query<(&mut Pillar, &mut Transform, &mut Mover)>,
) {
    if crate::game_state::is_playing(&game_state) && timer.0.tick(time.delta()).just_finished() {
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
