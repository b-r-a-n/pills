use bevy::prelude::*;
use super::*;
use pills_core::*;

pub(crate) struct LimitedPillAuraPlugin;

impl Plugin for LimitedPillAuraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, count_pills);
    }
}

#[derive(Component)]
pub struct LimitedPillPolicy {
    rem_pills: u32,
    handler: AuraEffect,
}

impl LimitedPillPolicy {
    pub fn new(max_pills: u32, handler: AuraEffect) -> Self {
        Self {
            rem_pills: max_pills,
            handler,
        }
    }
}

fn count_pills(
    mut commands: Commands,
    mut events: EventReader<BoardEvent>,
    mut policies: Query<(&mut LimitedPillPolicy, &InBoard)>
) {
    if events.is_empty() { return }
    for event in events.iter() {
        match event {
            BoardEvent::PillAdded(added) => {
                for (mut policy, board) in policies.iter_mut() {
                    if board.0 == added.board {
                        policy.rem_pills -= 1;
                        if policy.rem_pills == 0 {
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