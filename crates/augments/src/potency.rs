use super::*;

#[derive(Clone, Copy, Component)]
pub struct Potency {
    pub amount: u8,
    pub(crate) filter: fn((Option<&Pill>, Option<&Virus>)) -> bool,
}

pub(crate) fn apply(
    mut commands: Commands,
    augments: Query<(Entity, &Potency)>,
    pieces: Query<(Entity, &Pill)>,
) {
    for (augment_id, augment) in &augments {
        for (id, piece) in &pieces {
            if (augment.filter)((Some(piece), None)) {
                commands.entity(id).insert(RemoveStack(2));
            }
        }
        commands.entity(augment_id).despawn_recursive();
    }
}
