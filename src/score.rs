use crate::player::PlayerCrossedPillarEvent;
use bevy::prelude::*;

pub struct Score(u32);

pub struct IncreaseScoreEvent;
pub struct ResetScoreEvent;
pub struct ScoreUpdatedEvent(pub u32);

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Score(0))
            .add_event::<IncreaseScoreEvent>()
            .add_event::<ResetScoreEvent>()
            .add_event::<ScoreUpdatedEvent>()
            .add_system(scoring_system)
            .add_system(score_event_handler_system);
    }
}

fn scoring_system(
    mut crossed_event: EventReader<PlayerCrossedPillarEvent>,
    mut increase_score_events: EventWriter<IncreaseScoreEvent>,
) {
    crossed_event.iter().for_each(|_| {
        increase_score_events.send(IncreaseScoreEvent);
    });
}

fn score_event_handler_system(
    mut score: ResMut<Score>,
    mut increase_score_events: EventReader<IncreaseScoreEvent>,
    mut reset_score_events: EventReader<ResetScoreEvent>,
    mut score_updated_events: EventWriter<ScoreUpdatedEvent>,
) {
    let old_score = score.0;

    increase_score_events.iter().for_each(|_| {
        score.0 += 1;
    });

    reset_score_events.iter().for_each(|_| {
        score.0 = 0;
    });

    if old_score != score.0 {
        score_updated_events.send(ScoreUpdatedEvent(score.0));
    }
}
