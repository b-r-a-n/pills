use bevy::prelude::*;
use pills_game_board::CellColor;

#[derive(Component)]
pub struct BoardPosition {
    pub row: u8,
    pub column: u8,
}

#[derive(Clone, Copy, Component)]
pub struct Virus(pub CellColor);

#[derive(Clone, Copy, Component)]
pub struct Pill(pub CellColor);

#[derive(Clone, Copy, Component)]
pub struct ClearedCell {
    pub color: CellColor,
    pub was_virus: bool,
}

#[derive(Component)]
pub struct NextPill(pub u8);