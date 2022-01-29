use bevy::prelude::*;

#[derive(Component)]
struct Player {
    y_velocity: f32,
    dead: bool,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(simulate_player_gravity)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("player.png"),
            ..Default::default()
        })
        .insert(Player {
            y_velocity: 0.0,
            dead: false,
        });
}

fn simulate_player_gravity(
    windows: Res<Windows>,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    let (mut player, mut transform) = query.single_mut();

    if !player.dead {
        if keyboard_input.just_pressed(KeyCode::Space) {
            player.y_velocity = 5.0;
        } else {
            player.y_velocity -= time.delta().as_secs_f32() * 9.81;
        }

        transform.translation.y += player.y_velocity;

        let window = windows.get_primary().unwrap();

        let (min_y, max_y) = (-window.height() as f32 / 2.0, window.height() as f32 / 2.0);

        if transform.translation.y < min_y || transform.translation.y > max_y {
            player.dead = true;
        }
    }
}
