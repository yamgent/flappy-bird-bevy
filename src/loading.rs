use bevy::prelude::*;

use crate::game_state::GameState;

pub struct LoadingAssets(pub Vec<HandleUntyped>);

pub struct LoadingManagerPlugin;

impl Plugin for LoadingManagerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LoadingAssets(vec![]))
            .add_event::<FinishLoadingEvent>()
            .add_startup_system(setup_loading)
            .add_system(check_loading_system);
    }
}

pub struct FinishLoadingEvent;

#[derive(Component)]
struct LoadingText;

fn setup_loading(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<LoadingAssets>,
) {
    let font = asset_server.load("FiraSans-Bold.ttf");

    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "Loading...".to_string(),
                    style: TextStyle {
                        font: font.clone(),
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
            ..Default::default()
        })
        .insert(LoadingText);

    loading.0.push(font.clone_untyped());
}

fn check_loading_system(
    game_status: Res<GameState>,
    mut query: Query<(&mut Text, &mut Visibility), With<LoadingText>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    loading: Option<Res<LoadingAssets>>,
    mut finish_loading_events: EventWriter<FinishLoadingEvent>,
) {
    if crate::game_state::is_loading(&game_status) {
        use bevy::asset::LoadState;

        match asset_server.get_group_load_state(loading.unwrap().0.iter().map(|h| h.id)) {
            LoadState::Failed => {
                query.iter_mut().for_each(|(mut text, _)| {
                    text.sections[0].value = "Loading failed...".to_string();
                });
            }
            LoadState::Loaded => {
                finish_loading_events.send(FinishLoadingEvent);

                query.iter_mut().for_each(|(_, mut visibility)| {
                    visibility.is_visible = false;
                });

                commands.remove_resource::<LoadingAssets>();
            }
            _ => {}
        }
    }
}
