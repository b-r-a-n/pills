pub use aura::*;
use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
pub use score_aura::*;

mod aura;
mod score_aura;

pub struct AuraPluginGroup;

impl PluginGroup for AuraPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(AuraPlugin)
            .add(ScoreAuraPlugin)
    }
}