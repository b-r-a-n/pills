use bevy::prelude::*;
use pills_core::*;
use super::*;

pub(crate) struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Paused),  setup)
            .add_systems(Update, handle_interactions.run_if(in_state(AppState::PauseMenu)))
            .add_systems(OnEnter(AppState::PauseMenu), spawn)
            .add_systems(OnExit(AppState::PauseMenu),  despawn)
        ;
    }
}

fn setup(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
) {
    commands.spawn(MenuTitle::Custom("Paused".to_string()));
    commands.spawn(MenuOption::Play);
    state.set(AppState::PauseMenu);
}