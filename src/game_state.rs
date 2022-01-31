use bevy::prelude::*;

use crate::{loading::FinishLoadingEvent, PlayerKilledEvent};

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameState(GameStateType::Loading))
            .add_event::<StartNewGameEvent>()
            .add_event::<OnGameStateChangedEvent>()
            .add_system(finish_loading_system)
            .add_system(start_new_system)
            .add_system(game_over_system);
    }
}

pub struct StartNewGameEvent;
pub struct OnGameStateChangedEvent(pub GameStateType);

#[derive(Clone, Copy)] // TODO: Remove this when event is no longer using GameState?
pub enum GameStateType {
    Loading,
    StartScreen,
    Playing,
    GameOver,
}

pub struct GameState(pub GameStateType);

// TODO: Provide this as methods of GameStateType?
pub fn is_playing(game_state: &Res<GameState>) -> bool {
    matches!(game_state.0, GameStateType::Playing)
}

pub fn is_game_over(game_state: &Res<GameState>) -> bool {
    matches!(game_state.0, GameStateType::GameOver)
}

pub fn is_loading(game_state: &Res<GameState>) -> bool {
    matches!(game_state.0, GameStateType::Loading)
}

fn update_game_state(
    new_state: GameStateType,
    game_status: &mut ResMut<GameState>,
    on_change_event: &mut EventWriter<OnGameStateChangedEvent>,
) {
    game_status.0 = new_state;
    on_change_event.send(OnGameStateChangedEvent(new_state));
}

fn finish_loading_system(
    mut game_status: ResMut<GameState>,
    mut finish_loading_events: EventReader<FinishLoadingEvent>,
    mut on_change_event: EventWriter<OnGameStateChangedEvent>,
) {
    finish_loading_events.iter().for_each(|_| {
        update_game_state(
            GameStateType::StartScreen,
            &mut game_status,
            &mut on_change_event,
        );
    });
}

fn start_new_system(
    mut game_status: ResMut<GameState>,
    mut start_events: EventReader<StartNewGameEvent>,
    mut on_change_event: EventWriter<OnGameStateChangedEvent>,
) {
    start_events.iter().for_each(|_| {
        update_game_state(
            GameStateType::Playing,
            &mut game_status,
            &mut on_change_event,
        );
    });
}

fn game_over_system(
    mut game_status: ResMut<GameState>,
    mut killed_events: EventReader<PlayerKilledEvent>,
    mut on_change_event: EventWriter<OnGameStateChangedEvent>,
) {
    killed_events.iter().for_each(|_| {
        update_game_state(
            GameStateType::GameOver,
            &mut game_status,
            &mut on_change_event,
        );
    });
}
