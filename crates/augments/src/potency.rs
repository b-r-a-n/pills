use super::*;

#[derive(Clone, Copy, Component, Debug)]
pub struct Potency {
    pub amount: u8,
    pub(crate) filter: fn((Option<&Pill>, Option<&Virus>)) -> bool,
}

pub(crate) fn apply(
    augments: Query<&Potency>,
    mut pieces: Query<(&Pill, &mut RemoveStack), Added<Pill>>,
) {
    for augment in &augments {
        for (piece, mut remove_stack) in &mut pieces {
            if (augment.filter)((Some(piece), None)) {
                remove_stack.0 += augment.amount as usize;
            }
        }
    }
}
