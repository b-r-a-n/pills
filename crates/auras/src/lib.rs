pub use aura::*;
use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
pub use score_aura::*;
pub use limited_move_aura::*;
pub use limited_pill_aura::*;

mod aura;
mod score_aura;
mod limited_move_aura;
mod limited_pill_aura;

pub struct AuraPluginGroup;

impl PluginGroup for AuraPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(AuraPlugin)
            .add(ScoreAuraPlugin)
            .add(LimitedMoveAuraPlugin)
            .add(LimitedPillAuraPlugin)
    }
}
