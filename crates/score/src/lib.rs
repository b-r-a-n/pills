use bevy::prelude::*;
use bevy::sprite::Anchor;
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
                (animate_floating_scores)
                    .run_if(in_state(GameState::Active)))
            .add_systems(
                PostUpdate,
                (
                    apply_score_changes,
                    update_score_board.after(apply_score_changes))
                        .run_if(in_state(GameState::Active)))
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
    pub position: Option<(u8, u8)>,
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
    query: Query<(Entity, Option<&ScorePolicy>, Option<&BoardInfoContainer>, Option<&BoardPlayer>), (With<GameBoard>, Without<Score>)>,
    player_score: Query<&GlobalScore>,
) {
    for (entity, maybe_policy, maybe_container, maybe_player) in query.iter() {
        info!("Adding score tracking to {:?}", entity);
        commands.entity(entity)
            .insert(Score(0))
        ;
        if maybe_policy.is_none() {
            commands.entity(entity)
                .insert(ScorePolicy::default());
        }
        if let Some(container) = maybe_container {
            // Add a scoreboard to the container
            let score_board_ent = commands.spawn((
                Text2dBundle {
                    text: Text::from_section(
                        "Score: 0".to_string(),
                        TextStyle {font_size: 32.0, color: Color::WHITE, ..default()}
                    ),
                    text_anchor: Anchor::TopLeft,
                    transform: Transform::from_xyz(0.0, 0.0, 1.0),
                    ..default()
                },
            ))
                .set_parent(container.0)
                .id()
            ;
            // Add a component to the board entity that points to the scoreboard
            commands.entity(entity)
                .insert(ScoreBoard(score_board_ent));
        }
        if let Some(player) = maybe_player {
            if player_score.get(player.0).is_err() {
                commands.entity(player.0)
                    .insert(GlobalScore(0));
            }
        }
    }
}

fn update_global_score(
    mut board_scores: Query<(&BoardPlayer, &mut Score)>,
    mut global_scores: Query<&mut GlobalScore>,
    level: Res<Level>,
) {
    for entity in level.board_configs.iter() {
        if let Ok((player, mut score)) = board_scores.get_mut(*entity) {
            if let Ok(mut global_score) = global_scores.get_mut(player.0) {
                global_score.0 += score.0;
            }
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
                        position: None,
                        amount:f(&added.pill)
                    });
                }
            },
            BoardEvent::VirusRemoved(removed) => {
                if let Ok(policy) = policies.get(removed.board) {
                    let f = policy.virus_removed;
                    commands.spawn(ScoreChange{
                        score_entity: removed.board, 
                        source_entity: removed.piece,
                        position: Some((removed.row, removed.col)),
                        amount:f(&removed.virus)
                    });
                }
            },
            _ => {},
        }
    }
}

fn apply_score_changes(
    mut commands: Commands,
    mut scores: Query<&mut Score>,
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
            info!("Want to spawn floating text for {:?}", change.source_entity);
            if let Some(position) = change.position {
                info!("Spawning floating text at {:?}", position);
                commands.spawn((
                    Text2dBundle {
                        text: Text::from_section(
                            format!("{}", actual_amount).to_string(),
                            TextStyle {font_size: 32.0, color: text_color, ..default()}
                        ),
                        ..default()
                    },
                    DespawnIn(Timer::from_seconds(1.5, TimerMode::Once)),
                    FloatingScoreText,
                    BoardPosition { row: position.0, column: position.1 },
                    InBoard(change.score_entity),
                ));
            }
        }
        commands.entity(entity).remove::<ScoreChange>();
    }
}