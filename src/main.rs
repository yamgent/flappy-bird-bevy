use bevy::prelude::*;

#[derive(Component)]
struct Player {
    y_velocity: f32,
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
        .insert(Player { y_velocity: 0.0 });
}

fn simulate_player_gravity(time: Res<Time>, mut query: Query<(&mut Player, &mut Transform)>) {
    let (mut player, mut transform) = query.single_mut();

    player.y_velocity -= time.delta().as_secs_f32() * 9.81;
    transform.translation.y += player.y_velocity;
}
