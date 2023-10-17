use super::*;

#[derive(Clone, Copy, Component, Debug)]
pub struct Resilience {
    pub amount: u8,
    pub(crate) filter: fn((Option<&Pill>, Option<&Virus>)) -> bool,
}

pub(crate) fn apply(
    augments: Query<&Resilience>,
    mut pieces: Query<(AnyOf<(&Pill, &Virus)>, &mut Stacked), Or<(Added<Pill>, Added<Virus>)>>,
) {
    for augment in &augments {
        for (piece, mut stacked) in &mut pieces {
            if (augment.filter)(piece) {
                stacked.0 += augment.amount as usize;
            }
        }
    }
}
