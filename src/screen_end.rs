use bevy::prelude::*;

use crate::game_state::{GameState, GameStateType, OnGameStateChangedEvent, StartNewGameEvent};

pub struct ScreenEndPlugin;

impl Plugin for ScreenEndPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(game_over_system)
            .add_system(end_screen_input_system);
    }
}

#[derive(Component)]
struct ScreenEndText;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("FiraSans-Bold.ttf");

    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: "Game Over!".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 60.0,
                            color: Color::RED,
                        },
                    },
                    TextSection {
                        value: "  Press <R> to restart".to_string(),
                        style: TextStyle {
                            font,
                            font_size: 40.0,
                            color: Color::BLACK,
                        },
                    },
                ],
                alignment: TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                },
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Percent(50.0),
                    left: Val::Percent(50.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(ScreenEndText);
}

fn game_over_system(
    mut game_status_changed: EventReader<OnGameStateChangedEvent>,
    mut game_over_query: Query<&mut Visibility, With<ScreenEndText>>,
) {
    game_status_changed.iter().for_each(|event| {
        let mut game_over_visibility = game_over_query.single_mut();
        game_over_visibility.is_visible = matches!(event.0, GameStateType::GameOver);
    });
}

fn end_screen_input_system(
    game_state: Res<GameState>,
    keyboard_input: Res<Input<KeyCode>>,
    mut start_new_events: EventWriter<StartNewGameEvent>,
) {
    if crate::game_state::is_game_over(&game_state) && keyboard_input.just_pressed(KeyCode::R) {
        start_new_events.send(StartNewGameEvent);
    }
}
