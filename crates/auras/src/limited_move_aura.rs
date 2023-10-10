use bevy::prelude::*;
use super::*;
use pills_core::*;

#[derive(Component)]
pub struct MovesContainer;

pub(crate) struct LimitedMoveAuraPlugin;

#[derive(Component)]
pub struct LimitedMovePolicy {
    max_moves: u32,
    rem_moves: u32,
    handler: AuraEffect,
}

impl LimitedMovePolicy {
    pub fn new(max_moves: u32, handler: AuraEffect) -> Self {
        Self {
            max_moves,
            rem_moves: max_moves,
            handler,
        }
    }
}

impl Plugin for LimitedMoveAuraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (display_remaining, count_moves));
    }
}

fn display_remaining(
    mut commands: Commands,
    mut policies: Query<(Entity, &LimitedMovePolicy, Option<&mut Text>, &InBoard), Changed<LimitedMovePolicy>>,
) {
    for (entity, policy, maybe_text, board) in policies.iter_mut() {
        let str = format!("Moves: {}/{}", policy.rem_moves, policy.max_moves);
        if let Some(mut text) = maybe_text {
            text.sections[0].value = str;
        } else {
            info!("Injecting text for remaining moves");
            commands
                .entity(entity)
                .insert(Text2dBundle {
                    text: Text::from_section(
                        str, 
                        TextStyle {font_size: 20.0, color: Color::WHITE, ..default()}
                    ),
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                    ..default()
                })
                .set_parent(**board)
            ;
        }
    }
}

fn count_moves(
    mut commands: Commands,
    mut events: EventReader<BoardEvent>,
    mut policies: Query<(&mut LimitedMovePolicy, &InBoard)>
) {
    if events.is_empty() { return }
    for event in events.iter() {
        match event {
            BoardEvent::PillMoved(movement) => {
                for (mut policy, board) in policies.iter_mut() {
                    if board.0 == movement.board {
                        policy.rem_moves -= 1;
                        if policy.rem_moves == 0 {
                            match &policy.handler {
                                AuraEffect::BoardFinished(result) => {
                                    commands.entity(board.0).insert(*result);
                                }
                            }
                        }
                    }
                }
            },
            _ => {},
        }
    }
}