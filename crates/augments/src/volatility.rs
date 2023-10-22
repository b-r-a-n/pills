use super::*;

#[derive(Clone, Copy, Component, Debug)]
pub struct Volatility {
    pub area: AreaOfEffect,
    pub filter: fn((Option<&Pill>, Option<&Virus>)) -> bool,
}

pub(crate) fn apply(
    augments: Query<&Volatility>,
    mut pieces: Query<(AnyOf<(&Pill, &Virus)>, &mut Explosive), Or<(Added<Pill>, Added<Virus>)>>,
) {
    for augment in &augments {
        for (piece, mut explosive) in &mut pieces {
            if (augment.filter)(piece) {
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
