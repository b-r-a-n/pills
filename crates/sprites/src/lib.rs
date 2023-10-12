use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use board::*;
use pieces::*;

mod board;
mod pieces;

pub const CELL_SIZE: f32 = 32.0;

pub struct PillsSpritesPluginGroup;

impl PluginGroup for PillsSpritesPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(BoardSpritesPlugin)
            .add(PieceSpritesPlugin)
    }
}