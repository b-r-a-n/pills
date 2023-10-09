use bevy::prelude::*;
use pills_pieces::*;
use super::*;

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
    mut events: EventReader<AuraEvent>,
    mut policies: Query<(&mut LimitedPillPolicy, &InBoard)>
) {
    if events.is_empty() { return }
    for event in events.iter() {
        match event {
            AuraEvent::PillAdded(b, _, _) => {
                for (mut policy, board) in policies.iter_mut() {
                    if board.0 == *b {
                        policy.rem_pills -= 1;
                        if policy.rem_pills == 0 {
                            match &policy.handler {
                                AuraEffect::BoardFinished(result) => {
                                    commands.entity(*b).insert(*result);
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