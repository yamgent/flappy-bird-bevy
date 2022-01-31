// TODO: Move everything here
use bevy::prelude::*;

pub struct LoadingAssets(pub Vec<HandleUntyped>);

pub struct LoadingManagerPlugin;

impl Plugin for LoadingManagerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LoadingAssets(vec![]))
            .add_event::<FinishLoadingEvent>();
    }
}

pub struct FinishLoadingEvent;
