use bevy::prelude::*;

pub struct GameCorePlugin;

impl Plugin for GameCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}

fn setup(mut commands: Commands, mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    window.set_resizable(false);

    println!("Window size: {} {}", window.width(), window.height());

    let mut camera_bundle = OrthographicCameraBundle::new_2d();

    // the default makes it such that negative z is clipped
    // we need to use z: -1 for the background, so shift the camera a bit more forward
    camera_bundle.transform.translation.z = 500.0;

    commands.spawn_bundle(camera_bundle);
    commands.spawn_bundle(UiCameraBundle::default());
}
