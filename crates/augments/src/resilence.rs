use super::*;

#[derive(Clone, Copy, Component, Debug)]
pub struct Resilience {
    pub amount: u8,
    pub(crate) filter: fn((Option<&Pill>, Option<&Virus>)) -> bool,
}

pub(crate) fn apply(
    augments: Query<(&Resilience, &InBoard)>,
    mut pieces: Query<(AnyOf<(&Pill, &Virus)>, &mut Stacked, &InBoard), Or<(Added<Pill>, Added<Virus>)>>,
) {
    for (augment, augment_board_id) in &augments {
        for (piece, mut stacked, piece_board_id) in &mut pieces {
            if **augment_board_id == **piece_board_id && (augment.filter)(piece) {
                stacked.0 += augment.amount as usize;
            }
        }
    }
}
