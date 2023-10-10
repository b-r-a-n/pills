use bevy::prelude::*;
use pills_core::*;
use pills_level::*;
pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                OnTransition { from: GameState::Starting, to: GameState::Active}, 
                add_score_tracking)
            .add_systems(
                OnTransition { from: GameState::Active, to: GameState::Finished }, 
                update_global_score)
            .add_systems(
                PreUpdate, 
                insert_score_changes
                    .run_if(in_state(GameState::Active)))
            .add_systems(
                Update, 
                (animate_floating_scores, apply_score_changes)
                    .run_if(in_state(GameState::Active)))
            .add_systems(
                PostUpdate,
                update_score_board.run_if(in_state(GameState::Active)))
        ;
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
pub struct ScorePolicy {
    virus_removed: fn(&Virus) -> i32,
    pill_added: fn(&Pill) -> i32,
}

impl Default for ScorePolicy {
    fn default() -> Self {
        Self {
            virus_removed: |_| 1,
            pill_added: |_| 0,
        }
    }
}

#[derive(Component)]
pub struct ScoreBoard(pub Entity);

fn add_score_tracking(
    mut commands: Commands,
    query: Query<(Entity, Option<&ScorePolicy>), (With<GameBoard>, Without<Score>, Without<GlobalScore>)>
) {
    for (entity, maybe_policy) in query.iter() {
        info!("Adding score tracking to {:?}", entity);
        commands.entity(entity)
            .insert(Score(0))
            .insert(GlobalScore(0));
        if maybe_policy.is_none() {
            commands.entity(entity)
                .insert(ScorePolicy::default());
        }
    }
}

fn update_global_score(
    mut scores: Query<(&mut Score, &mut GlobalScore, Option<&ScoreBoard>)>,
    mut score_boards: Query<&mut Text>,
    level: Res<Level>,
) {
    for entity in level.board_configs.iter() {
        if let Ok((mut score, mut global_score, maybe_score_board)) = scores.get_mut(*entity) {
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
        transform.translation.z = 1000.0;
    }
}

fn update_score_board(
    changed_scores: Query<(&Score, &ScoreBoard), Changed<Score>>,
    mut texts: Query<&mut Text>,
) {
    for (score, score_board) in changed_scores.iter() {
        if let Ok(mut text) = texts.get_mut(score_board.0) {
            text.sections[0].value = format!("Score: {}", score.0).to_string();
        }
    }
}

fn insert_score_changes(
    mut commands: Commands,
    mut events: EventReader<BoardEvent>,
    policies: Query<&ScorePolicy>,
) {
    for event in events.iter() {
        match event {
            BoardEvent::PillAdded(added) => {
                if let Ok(policy) = policies.get(added.board) {
                    let f = policy.pill_added;
                    commands.spawn(ScoreChange{
                        score_entity: added.board, 
                        source_entity: added.piece,
                        amount:f(&added.pill)
                    });
                    info!("Score change for adding a pill: {}", f(&added.pill));
                }
            },
            BoardEvent::VirusRemoved(removed) => {
                if let Ok(policy) = policies.get(removed.board) {
                    let f = policy.virus_removed;
                    commands.spawn(ScoreChange{
                        score_entity: removed.board, 
                        source_entity: removed.piece,
                        amount:f(&removed.virus)
                    });
                    info!("Score change for removing a virus: {}", f(&removed.virus));
                }
            },
            _ => {},
        }
    }
}

fn apply_score_changes(
    mut commands: Commands,
    mut scores: Query<&mut Score>,
    global_transforms: Query<&GlobalTransform, With<BoardPosition>>,
    score_changes: Query<(Entity, &ScoreChange)>
) {
    for (entity, change) in score_changes.iter() {
        if let Ok(mut score) = scores.get_mut(change.score_entity) {
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
            if actual_amount == 0 {
                continue;
            }
            // Spawn a floating text at the source entity position
            if let Ok(global_transform) = global_transforms.get(change.source_entity) {
                info!("Spawning floating text at {:?}", global_transform.translation());
                commands.spawn((
                    Text2dBundle {
                        text: Text::from_section(
                            format!("{}", actual_amount).to_string(),
                            TextStyle {font_size: CELL_SIZE, color: text_color, ..default()}
                        ),
                        transform: Transform::from_translation(global_transform.translation()),
                        ..default()
                    },
                    DespawnIn(Timer::from_seconds(1.5, TimerMode::Once)),
                    FloatingScoreText,
                ));
            }
        }
        commands.entity(entity).remove::<ScoreChange>();
    }
}