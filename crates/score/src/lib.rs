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

#[derive(Component)]
pub struct ScoreBoard(pub Entity);

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
    mut scores: Query<(&mut Score, &mut GlobalScore, Option<&ScoreBoard>)>,
    mut score_boards: Query<&mut Text>,
    mut events: EventReader<BoardResult>,
) {
    for event in events.iter() {
        if let Ok((mut score, mut global_score, maybe_score_board)) = scores.get_mut(event.0) {
            global_score.0 += score.0;
            score.0 = 0;
            if let Some(score_board_ent) = maybe_score_board {
                if let Ok(mut text) = score_boards.get_mut(score_board_ent.0) {
                    text.sections[0].value = "Score: 0".to_string();
                }
            }
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
    mut scores: Query<(&mut Score, Option<&ScoreBoard>)>,
    mut score_boards: Query<&mut Text>,
    positions: Query<Entity, With<BoardPosition>>,
    score_changes: Query<(Entity, &ScoreChange)>
) {
    for (entity, change) in score_changes.iter() {
        if let Ok((mut score, maybe_score_board)) = scores.get_mut(change.score_entity) {
            let mut text_color = Color::WHITE;
            let mut actual_amount = change.amount;
            if change.amount < 0 {
                if change.amount.abs() as usize > score.0 {
                    actual_amount = -(score.0 as i32);
                    score.0 = 0;
                } else {
                    score.0 -= change.amount.abs() as usize;
                }
                text_color = Color::RED;
            } else {
                score.0 += change.amount as usize;
                if change.amount > 0 {
                    text_color = Color::GREEN;
                }
            }
            if let Some(score_board) = maybe_score_board {
                if let Ok(mut text) = score_boards.get_mut(score_board.0) {
                    text.sections[0].value = format!("Score: {}", score.0).to_string();
                }
            }
            if actual_amount == 0 {
                continue;
            }
            // Spawn a floating text at the source entity position
            if let Ok(parent) = positions.get(change.source_entity) {
                commands.entity(parent).with_children(|builder| {
                    builder.spawn((
                        Text2dBundle {
                            text: Text::from_section(
                                format!("{}", actual_amount).to_string(),
                                TextStyle {font_size: 64.0, color: text_color, ..default()}
                            ),
                            ..default()
                        },
                        DespawnIn(Timer::from_seconds(1.5, TimerMode::Once)),
                        FloatingScoreText,
                    ));
                });
            }
        }
        commands.entity(entity).remove::<ScoreChange>();
    }
}