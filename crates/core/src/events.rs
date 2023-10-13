use bevy::prelude::*;
use crate::{Move, Pill, Rotate, Virus};

pub(crate) struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BoardEvent>()
        ;
    }
}

#[derive(Event, Debug)]
pub struct ClearEvent(Entity);

#[derive(Debug)]
pub struct PillAdded {
    pub board: Entity,
    pub piece: Entity,
    pub pill: Pill,
}

#[derive(Debug)]
pub struct VirusRemoved {
    pub board: Entity,
    pub piece: Entity,
    pub virus: Virus,
    pub row: u8,
    pub col: u8,
}

#[derive(Debug)]
pub enum Movement {
    Direction(Move),
    Rotation(Rotate),
}
#[derive(Debug)]
pub struct PillMoved {
    pub board: Entity,
    pub piece: Entity,
    pub movement: Movement,
}

#[derive(Debug)]
pub struct CellsCleared {
    pub board: Entity,
    pub count: usize,
}

#[derive(Debug, Event)]
pub enum BoardEvent {
    PillAdded(PillAdded),
    VirusRemoved(VirusRemoved),
    PillMoved(PillMoved),
    CellsCleared(CellsCleared),
}

impl Into<Movement> for Move {
    fn into(self) -> Movement {
        Movement::Direction(self)
    }
}

impl Into<Movement> for Rotate {
    fn into(self) -> Movement {
        Movement::Rotation(self)
    }
}

impl BoardEvent {
    pub(crate) fn pill_added(board: Entity, piece: Entity, pill: Pill) -> Self {
        Self::PillAdded(PillAdded { board, piece, pill })
    }

    pub(crate) fn virus_removed(board: Entity, piece: Entity, virus: Virus, row: u8, col: u8) -> Self {
        Self::VirusRemoved(VirusRemoved { board, piece, virus, row, col })
    }

    pub(crate) fn pill_moved(board: Entity, piece: Entity, movement: impl Into<Movement>) -> Self {
        let movement = movement.into();
        Self::PillMoved(PillMoved { board, piece, movement })
    }

    pub(crate) fn cells_cleared(board: Entity, count: usize) -> Self {
        Self::CellsCleared(CellsCleared { board, count })
    }
}