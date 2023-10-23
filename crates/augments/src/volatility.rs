use super::*;

#[derive(Clone, Copy, Component, Debug)]
pub struct Volatility {
    pub area: AreaOfEffect,
    pub filter: fn((Option<&Pill>, Option<&Virus>)) -> bool,
}

pub(crate) fn apply(
    augments: Query<(&Volatility, &InBoard)>,
    mut pieces: Query<(AnyOf<(&Pill, &Virus)>, &mut Explosive, &InBoard), Or<(Added<Pill>, Added<Virus>)>>,
) {
    for (augment, augment_board_id) in &augments {
        for (piece, mut explosive, piece_board_id) in &mut pieces {
            if **augment_board_id == **piece_board_id && (augment.filter)(piece) {
                match augment.area {
                    AreaOfEffect::Radius(radius) => { 
                        let current_radius = match explosive.0 { AreaOfEffect::Radius(r) => r, _ => 0 };
                        explosive.0 = AreaOfEffect::Radius(current_radius + radius);
                    },
                    _ => {}
                }
            }
        }
    }
}
