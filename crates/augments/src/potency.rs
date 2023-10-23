use super::*;

#[derive(Clone, Copy, Component, Debug)]
pub struct Potency {
    pub amount: u8,
    pub(crate) filter: fn((Option<&Pill>, Option<&Virus>)) -> bool,
}

pub(crate) fn apply(
    augments: Query<(&Potency, &InBoard)>,
    mut pieces: Query<(&Pill, &mut RemoveStack, &InBoard), Added<Pill>>,
) {
    for (augment, augment_board_id) in &augments {
        for (piece, mut remove_stack, piece_board_id) in &mut pieces {
            if **augment_board_id == **piece_board_id && (augment.filter)((Some(piece), None)) {
                remove_stack.0 += augment.amount as usize;
            }
        }
    }
}
