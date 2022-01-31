use crate::{
    loading::LoadingAssets,
    player::{PlayerCrossedPillarEvent, PlayerKilledEvent},
};
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioPlugin, AudioSource};

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin)
            .add_startup_system(setup_audio)
            .add_system(audio_system);
    }
}

struct AudioCollection {
    crossed: Handle<AudioSource>,
    dead: Handle<AudioSource>,
}

fn setup_audio(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<LoadingAssets>,
) {
    let crossed = asset_server.load("crossed.wav");
    let dead = asset_server.load("dead.wav");

    loading.0.push(crossed.clone_untyped());
    loading.0.push(dead.clone_untyped());

    commands.insert_resource(AudioCollection { crossed, dead });
}

fn audio_system(
    audio: Res<Audio>,
    audio_collection: Res<AudioCollection>,
    mut crossed_events: EventReader<PlayerCrossedPillarEvent>,
    mut killed_events: EventReader<PlayerKilledEvent>,
) {
    if crossed_events.iter().count() > 0 {
        audio.play(audio_collection.crossed.clone());
    };

    if killed_events.iter().count() > 0 {
        audio.play(audio_collection.dead.clone());
    }
}
