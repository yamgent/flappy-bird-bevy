use bevy::prelude::*;

use crate::game_state::GameState;

pub struct MoverPlugin;

impl Plugin for MoverPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(mover_system)
            .add_system(mover_window_left_despawn_bound_system);
    }
}

#[derive(Component)]
pub struct Mover {
    pub active: bool,
    pub velocity: Vec3,
    pub acceleration: Vec3,
}

#[derive(Component)]
pub struct MoverWindowLeftDespawnBound {
    pub object_width: f32,
}

fn mover_system(
    game_state: Res<GameState>,
    time: Res<Time>,
    mut query: Query<(&mut Mover, &mut Transform)>,
) {
    // TODO: Is the coupling with game_state reasonable?
    if crate::game_state::is_playing(&game_state) {
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
    game_state: Res<GameState>,
    windows: Res<Windows>,
    mut query: Query<(&MoverWindowLeftDespawnBound, &mut Mover, &mut Transform)>,
) {
    let window = windows.get_primary().unwrap();
    let window_width = window.width() as f32;

    if crate::game_state::is_playing(&game_state) {
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
