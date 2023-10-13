use super::*;

#[derive(Clone, Copy, Component)]
pub struct Resilience {
    pub amount: u8,
    pub(crate) filter: fn((Option<&Pill>, Option<&Virus>)) -> bool,
}

pub(crate) fn apply(
    mut commands: Commands,
    augments: Query<(Entity, &Resilience)>,
    pieces: Query<(Entity, AnyOf<(&Pill, &Virus)>)>,
) {
    for (augment_id, augment) in &augments {
        for (id, piece) in &pieces {
            if (augment.filter)(piece) {
                commands.entity(id).insert(Stacked(2));
            }
        }
        commands.entity(augment_id).despawn_recursive();
    }
}
