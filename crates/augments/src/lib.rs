use bevy::prelude::*;
use bevy::ecs::system::EntityCommand;
use pills_core::*;
use pills_game_board::CellColor;
use frequency::Frequency;
use resilence::Resilience;
use potency::Potency;
use volatility::Volatility;
use urgency::Urgency;
use rand::prelude::*;

mod frequency;
mod potency;
mod resilence;
mod urgency;
mod volatility;

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

fn all_blue((pill, virus): (Option<&Pill>, Option<&Virus>)) -> bool {
    return virus.map(|v| v.0 == CellColor::BLUE).unwrap_or(false) || pill.map(|p| p.0 == CellColor::BLUE).unwrap_or(false);
}

fn all_yellow((pill, virus): (Option<&Pill>, Option<&Virus>)) -> bool {
    return virus.map(|v| v.0 == CellColor::YELLOW).unwrap_or(false) || pill.map(|p| p.0 == CellColor::YELLOW).unwrap_or(false);
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

#[derive(Debug)]
pub enum Augment {
    Frequency(Frequency),
    Potency(Potency),
    Resilience(Resilience),
    Urgency(Urgency),
    Volatility(Volatility),
}

impl Augment {
    pub fn cost(&self) -> u32 {
        match self {
            Augment::Frequency(f) => { 1 },
            Augment::Potency(p) => if p.filter == all_pills { 3 } else { 1 },
            Augment::Resilience(r) => if r.filter == all_pills { 3 } else { 1 },
            Augment::Urgency(_) => 1,
            Augment::Volatility(v) => if v.filter == all_viruses { 3 } else { 1 },
        }
    }
}

impl EntityCommand for Augment {
    fn apply(self, id: Entity, world: &mut World) {
        info!("Applying augment {:?}", self);
        match self {
            Augment::Frequency(frequency) => {
                world.entity_mut(id).insert(frequency);
            },
            Augment::Potency(potency) => {
                world.entity_mut(id).insert(potency);
            },
            Augment::Resilience(resilience) => {
                world.entity_mut(id).insert(resilience);
            },
            Augment::Urgency(urgency) => {
                world.entity_mut(id).insert(urgency);
            }
            Augment::Volatility(volatility) => {
                world.entity_mut(id).insert(volatility);
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
    match rng.gen_range(0..=1) {
        0 => Augment::Potency(Potency {
            amount, 
            filter: [all_pills, red_pills, yellow_pills, blue_pills][rng.gen_range(0..=3)]
        }),
        1 => Augment::Volatility(Volatility { 
            area: AreaOfEffect::Radius(amount-1), 
            filter: [all_viruses, red_viruses, yellow_viruses, blue_viruses][rng.gen_range(0..=3)]
        }),
        _ => unreachable!()
    }
}

pub fn random_harmful_augment(rng: &mut ThreadRng) -> Augment {
    match rng.gen_range(0..=2) {
        0 => { 
            let filter = match rng.gen_range(0..=3) {
                0 => { all_viruses},
                1 => red_viruses,
                2 => yellow_viruses,
                3 => blue_viruses,
                _ => unreachable!()
            };
            let amount = 1;
            Augment::Resilience( Resilience { amount, filter}) 
        },
        1 => {
            let amount = 0.1;
            Augment::Urgency(Urgency { amount })
        },
        2 => {
            let amount = 10;
            Augment::Frequency(Frequency { amount })
        }
        _ => unreachable!()
    }
}

pub fn add_augment(
    commands: &mut Commands,
    (name, augments): (&str, &[&Augment])
) {
    info!("Adding augment: {}", name);
    for augment in augments {
        match augment {
            Augment::Frequency(frequency) => {
                commands.spawn(*frequency);
            },
            Augment::Potency(potency) => {
                commands.spawn(*potency);
            },
            Augment::Resilience(resilience) => {
                commands.spawn(*resilience);
            },
            Augment::Urgency(urgency) => {
                commands.spawn(*urgency);
            },
            Augment::Volatility(volatility) => {
                commands.spawn(*volatility);
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
                (frequency::apply, potency::apply, resilence::apply, urgency::apply, volatility::apply)
            );
    }
}