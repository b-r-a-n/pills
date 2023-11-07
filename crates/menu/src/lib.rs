use bevy::prelude::*;
use bevy::ecs::system::EntityCommand;
use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use pills_core::*;
use pills_level::*;
use pills_score::*;
use pills_augments::*;

use menu::*;
use main_menu::*;
use pause_menu::*;
use level_menu::*;

mod menu;
mod main_menu;
mod pause_menu;
mod level_menu;

pub struct MenuPluginGroup;

impl PluginGroup for MenuPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(LevelMenuPlugin)
            .add(MainMenuPlugin)
            .add(PauseMenuPlugin)
    }
}

#[derive(Resource)]
struct MenuData {
    root_entity: Entity,
}