use bevy::prelude::*;
use super::*;

pub(crate) struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup)
            .add_systems(Update, handle_interactions.run_if(in_state(AppState::MainMenu)))
            .add_systems(OnEnter(AppState::MainMenu), spawn)
            .add_systems(OnExit(AppState::MainMenu), despawn)
        ;
    }
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn(
        MenuTitle::Custom("Pills".to_string())
    );
    commands.spawn_batch([
        (MenuOption::Play),
        (MenuOption::Exit),
    ]);
}