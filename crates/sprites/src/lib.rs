use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use board::*;
mod board;

pub struct PillsSpritesPluginGroup;

impl PluginGroup for PillsSpritesPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(BoardSpritesPlugin)
    }
}