use bevy::prelude::*;
use pills_core::*;
pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PreUpdate, add_score_tracking)
            .add_systems(Update, (animate_floating_scores, update_score, update_global_score.after(update_score)));
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct Score(pub usize);

#[derive(Component)]
struct FloatingScoreText;

#[derive(Component, Deref, DerefMut, Resource)]
pub struct GlobalScore(pub usize);

#[derive(Component)]
pub struct ScoreChange {
    pub score_entity: Entity, 
    pub source_entity: Entity,
    pub amount: i32
}

fn add_score_tracking(
    mut commands: Commands,
    query: Query<Entity, (With<GameBoard>, Without<Score>, Without<GlobalScore>)>
) {
    for entity in query.iter() {
        info!("Adding score tracking to {:?}", entity);
        commands.entity(entity)
            .insert(Score(0))
            .insert(GlobalScore(0));
    }
}

pub fn update_global_score(
    mut scores: Query<(&mut Score, &mut GlobalScore)>,
    mut events: EventReader<BoardResult>,
) {
    for event in events.iter() {
        if let Ok((mut score, mut global_score)) = scores.get_mut(event.0) {
            info!("Updating global score: {} + {}", global_score.0, score.0);
            global_score.0 += score.0;
            score.0 = 0;
        }
    }
}

fn animate_floating_scores(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<FloatingScoreText>>,
) {
    for mut transform in query.iter_mut() {
        transform.translation.y += time.delta_seconds() * 100.0;
        transform.translation.z = 1.0;
    }
}

fn update_score(
    mut commands: Commands,
    mut scores: Query<&mut Score>,
    positions: Query<Entity, With<BoardPosition>>,
    score_changes: Query<(Entity, &ScoreChange)>
) {
    for (entity, change) in score_changes.iter() {
        if let Ok(mut score) = scores.get_mut(change.score_entity) {
            let mut text_color = Color::WHITE;
            if change.amount < 0 {
                if change.amount.abs() as usize > score.0 {
                    score.0 = 0;
                } else {
                    score.0 -= change.amount.abs() as usize;
                    text_color = Color::RED;
                }
            } else {
                score.0 += change.amount as usize;
                text_color = Color::GREEN;
            }
            // Spawn a red text at the source entity position
            if let Ok(parent) = positions.get(change.source_entity) {
                commands.entity(parent).with_children(|builder| {
                    builder.spawn((
                        Text2dBundle {
                            text: Text::from_section(
                                format!("{}", change.amount).to_string(),
                                TextStyle {font_size: 64.0, color: text_color, ..default()}
                            ),
                            ..default()
                        },
                        FloatingScoreText,
                    ));
                });
            }
        }
        commands.entity(entity).remove::<ScoreChange>();
    }
}