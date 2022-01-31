use bevy::prelude::*;

pub struct PlayerCrossedPillarEvent;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerCrossedPillarEvent>();
    }
}
