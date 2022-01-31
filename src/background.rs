use bevy::prelude::*;

use crate::loading::LoadingAssets;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<LoadingAssets>,
) {
    let background = asset_server.load("background.png");
    loading.0.push(background.clone_untyped());

    commands.spawn_bundle(SpriteBundle {
        texture: background,
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, -1.0),
            ..Default::default()
        },
        ..Default::default()
    });
}
