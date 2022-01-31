use bevy::prelude::*;

use crate::{
    game_state::{GameState, StartNewGameEvent},
    loading::FinishLoadingEvent,
};

pub struct ScreenStartPlugin;

impl Plugin for ScreenStartPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(load_finish_system)
            .add_system(start_screen_input_system);
    }
}

#[derive(Component)]
struct StartScreenText;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("FiraSans-Bold.ttf");

    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "Press <Space> to start".to_string(),
                    style: TextStyle {
                        font,
                        font_size: 60.0,
                        color: Color::BLACK,
                    },
                }],
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
        .insert(StartScreenText);
}

fn load_finish_system(
    mut finish_loading_events: EventReader<FinishLoadingEvent>,
    mut query: Query<&mut Visibility, With<StartScreenText>>,
) {
    if finish_loading_events.iter().count() > 0 {
        let mut visibility = query.single_mut();
        visibility.is_visible = true;
    }
}

fn start_screen_input_system(
    game_state: Res<GameState>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Visibility, With<StartScreenText>>,
    mut start_new_events: EventWriter<StartNewGameEvent>,
) {
    if crate::game_state::is_start_screen(&game_state)
        && keyboard_input.just_pressed(KeyCode::Space)
    {
        start_new_events.send(StartNewGameEvent);

        let mut visibility = query.single_mut();
        visibility.is_visible = false;
    }
}
