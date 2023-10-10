pub use aura::*;
use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
pub use limited_move_aura::*;
pub use limited_pill_aura::*;

mod aura;
mod limited_move_aura;
mod limited_pill_aura;

pub struct AuraPluginGroup;

impl PluginGroup for AuraPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(AuraPlugin)
            //.add(LimitedMoveAuraPlugin)
            //.add(LimitedPillAuraPlugin)
    }
}
