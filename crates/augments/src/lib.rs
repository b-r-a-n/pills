use bevy::prelude::*;
use pills_core::*;
use pills_game_board::CellColor;
use pills_ui::Tooltip;
use resilence::Resilience;
use potency::Potency;

mod potency;
mod resilence;

fn all_pieces((pill, virus): (Option<&Pill>, Option<&Virus>)) -> bool {
    return pill.is_some() || virus.is_some()
}

fn all_pills((pill, _): (Option<&Pill>, Option<&Virus>)) -> bool {
    return pill.is_some()
}

fn all_viruses((_, virus): (Option<&Pill>, Option<&Virus>)) -> bool {
    return virus.is_some()
}

fn all_red((pill, virus): (Option<&Pill>, Option<&Virus>)) -> bool {
    return virus.map(|v| v.0 == CellColor::RED).unwrap_or(false) || pill.map(|p| p.0 == CellColor::RED).unwrap_or(false);
}

enum Augment {
    Resilience(Resilience),
    Potency(Potency),
}

type AugmentInfo = (&'static str, &'static [&'static Augment]);

const OVERDOSE: AugmentInfo = (
    "Overdose",
    &[
        &Augment::Resilience(Resilience {amount: 2, filter: all_pills}),
        &Augment::Potency(Potency {amount: 2, filter: all_pills}),
    ],
);

const SUPERBUG: AugmentInfo = (
    "Superbugs",
    &[&Augment::Resilience(Resilience {amount: 2, filter: all_viruses})],
);

fn do_stuff_with_info((name, augments): (&str, &[&Augment])) {
    println!("{}: {:?}", name, augments.len());
}

fn fake_system() {
    do_stuff_with_info(OVERDOSE);
}

pub struct AugmentPlugin;

impl Plugin for AugmentPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update, 
                (resilence::apply, potency::apply)
            );
    }
}