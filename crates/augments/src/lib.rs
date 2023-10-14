use bevy::prelude::*;
use bevy::ecs::system::EntityCommand;
use pills_core::*;
use pills_game_board::CellColor;
use resilence::Resilience;
use potency::Potency;
use rand::prelude::*;

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

fn red_pills((pill, _): (Option<&Pill>, Option<&Virus>)) -> bool {
    return pill.map(|p| p.0 == CellColor::RED).unwrap_or(false);
}

fn blue_pills((pill, _): (Option<&Pill>, Option<&Virus>)) -> bool {
    return pill.map(|p| p.0 == CellColor::BLUE).unwrap_or(false);
}

fn yellow_pills((pill, _): (Option<&Pill>, Option<&Virus>)) -> bool {
    return pill.map(|p| p.0 == CellColor::YELLOW).unwrap_or(false);
}

fn red_viruses((_, virus): (Option<&Pill>, Option<&Virus>)) -> bool {
    return virus.map(|p| p.0 == CellColor::RED).unwrap_or(false);
}

fn blue_viruses((_, virus): (Option<&Pill>, Option<&Virus>)) -> bool {
    return virus.map(|p| p.0 == CellColor::BLUE).unwrap_or(false);
}

fn yellow_viruses((_, virus): (Option<&Pill>, Option<&Virus>)) -> bool {
    return virus.map(|p| p.0 == CellColor::YELLOW).unwrap_or(false);
}

pub enum Augment {
    Resilience(Resilience),
    Potency(Potency),
}

impl EntityCommand for Augment {
    fn apply(self, id: Entity, world: &mut World) {
        match self {
            Augment::Resilience(resilience) => {
                world.entity_mut(id).insert(resilience);
            },
            Augment::Potency(potency) => {
                world.entity_mut(id).insert(potency);
            },
        }
    }
}

type AugmentInfo = (&'static str, &'static [&'static Augment]);

pub const OVERDOSE: AugmentInfo = (
    "Overdose",
    &[
        &Augment::Resilience(Resilience {amount: 2, filter: all_pills}),
        &Augment::Potency(Potency {amount: 2, filter: all_pills}),
    ],
);

pub const SUPERBUG: AugmentInfo = (
    "Superbugs",
    &[&Augment::Resilience(Resilience {amount: 2, filter: all_viruses})],
);

pub fn random_helpful_augment(rng: &mut ThreadRng) -> Augment {
    let amount = rng.gen_range(2..=4);
    let filter = match rng.gen_range(0..=3) {
        0 => all_pills,
        1 => red_pills,
        2 => yellow_pills,
        3 => blue_pills,
        _ => unreachable!()
    };
    return Augment::Potency(Potency {amount, filter})
}

pub fn random_harmful_augment(rng: &mut ThreadRng) -> Augment {
    let amount = rng.gen_range(2..=4);
    let filter = match rng.gen_range(0..=3) {
        0 => all_viruses,
        1 => red_viruses,
        2 => yellow_viruses,
        3 => blue_viruses,
        _ => unreachable!()
    };
    return Augment::Resilience(Resilience {amount, filter})
}

pub fn add_augment(
    commands: &mut Commands,
    (name, augments): (&str, &[&Augment])
) {
    info!("Adding augment: {}", name);
    for augment in augments {
        match augment {
            Augment::Resilience(resilience) => {
                commands.spawn(*resilience);
            },
            Augment::Potency(potency) => {
                commands.spawn(*potency);
            },
        }
    }
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