use bevy::prelude::*;
use crate::aura::AuraEvent;
use pills_pieces::*;
use pills_score::*;

pub(crate) struct ScoreAuraPlugin;

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

impl Plugin for ScoreAuraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, generate_changes);
    }
}

fn generate_changes(
    mut commands: Commands,
    mut events: EventReader<AuraEvent>,
    policies: Query<&ScorePolicy>,
) {
    for event in events.iter() {
        match event {
            AuraEvent::PillAdded(b, p) => {
                if let Ok(policy) = policies.get(*b) {
                    let f = policy.pill_added;
                    commands.spawn(ScoreChange{
                        score_entity: *b, 
                        amount:f(p)
                    });
                    info!("Score change for adding a pill: {}", f(p));
                }
            },
            AuraEvent::VirusRemoved(b, v) => {
                if let Ok(policy) = policies.get(*b) {
                    let f = policy.virus_removed;
                    commands.spawn(ScoreChange{
                        score_entity: *b, 
                        amount:f(v)
                    });
                    info!("Score change for removing a virus: {}", f(v));
                }
            },
        }
    }
}
