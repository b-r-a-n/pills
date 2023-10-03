use bevy::prelude::*;
pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, update_score);
    }
}

#[derive(Component, Deref, DerefMut, Resource)]
pub struct Score(pub usize);

#[derive(Component)]
pub struct ScoreChange {
    pub score_entity: Entity, 
    pub amount: i32
}

fn update_score(
    mut commands: Commands,
    mut scores: Query<&mut Score>,
    score_changes: Query<(Entity, &ScoreChange)>
) {
    for (entity, change) in score_changes.iter() {
        if let Ok(mut score) = scores.get_mut(change.score_entity) {
            if change.amount < 0 {
                if change.amount.abs() as usize > score.0 {
                    score.0 = 0;
                } else {
                    score.0 -= change.amount.abs() as usize;
                }
            } else {
                score.0 += change.amount as usize;
            }
        }
        commands.entity(entity).remove::<ScoreChange>();
    }
}