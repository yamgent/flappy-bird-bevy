use bevy::prelude::*;

use crate::{
    game_state::{GameState, StartNewGameEvent},
    loading::LoadingAssets,
    mover::Mover,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerCrossedPillarEvent>()
            .add_event::<PlayerKilledEvent>()
            .add_startup_system(setup_player)
            .add_system(new_game_system)
            .add_system(player_input_system)
            .add_system(player_bounds_check_system);
    }
}

const PLAYER_GRAVITY: f32 = 9.81 * 60.0;
const LEAP_Y_VELOCITY: f32 = 5.0 * 60.0;

pub struct PlayerCrossedPillarEvent;
pub struct PlayerKilledEvent;

#[derive(Component)]
pub struct Player;

fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<LoadingAssets>,
) {
    let player = asset_server.load("player.png");
    loading.0.push(player.clone_untyped());

    commands
        .spawn_bundle(SpriteBundle {
            texture: player,
            ..Default::default()
        })
        .insert(Player)
        .insert(Mover {
            active: true,
            velocity: Vec3::ZERO,
            acceleration: Vec3::new(0.0, -PLAYER_GRAVITY, 0.0),
        });
}

fn player_input_system(
    game_status: Res<GameState>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Mover, With<Player>>,
) {
    let mut mover = query.single_mut();

    if crate::game_state::is_playing(&game_status) && keyboard_input.just_pressed(KeyCode::Space) {
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

    if crate::game_state::is_playing(&game_status) {
        let window = windows.get_primary().unwrap();

        let (min_y, max_y) = (-window.height() as f32 / 2.0, window.height() as f32 / 2.0);

        if transform.translation.y < min_y || transform.translation.y > max_y {
            killed_event.send(PlayerKilledEvent);
        }
    }
}

fn new_game_system(
    mut query: Query<(&mut Transform, &mut Mover), With<Player>>,
    mut new_game_events: EventReader<StartNewGameEvent>,
) {
    if new_game_events.iter().count() > 0 {
        let (mut transform, mut mover) = query.single_mut();

        transform.translation = Vec3::ZERO;
        mover.velocity = Vec3::ZERO;
    }
}
