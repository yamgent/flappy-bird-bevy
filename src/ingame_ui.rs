use bevy::prelude::*;

use crate::score::ScoreUpdatedEvent;

pub struct IngameUiPlugin;

impl Plugin for IngameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_ingame_ui)
            .add_system(score_ui_update_system);
    }
}

#[derive(Component)]
struct ScoreText;

fn setup_ingame_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("FiraSans-Bold.ttf");

    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: "Score: ".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 30.0,
                            color: Color::BLACK,
                        },
                    },
                    TextSection {
                        value: "0".to_string(),
                        style: TextStyle {
                            font,
                            font_size: 30.0,
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
                    top: Val::Px(15.0),
                    left: Val::Px(15.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(ScoreText);
}

fn score_ui_update_system(
    mut query: Query<&mut Text, With<ScoreText>>,
    mut score_updated_events: EventReader<ScoreUpdatedEvent>,
) {
    let mut text = query.single_mut();

    score_updated_events.iter().for_each(|event| {
        text.sections[1].value = event.0.to_string();
    });
}
