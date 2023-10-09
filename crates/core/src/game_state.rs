use bevy::prelude::*;
use super::*;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum GameState {
    #[default]
    NotStarted,
    Starting,
    Active,
    Paused,
    Finished,
}

pub(crate) fn start_game(
    mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
    query: Query<(Entity, &BoardConfig, Option<&VirusSpawner>), Without<GameBoard>>,
) {
    for (entity, config, maybe_spawner) in query.iter() {
        info!("[start_game] Adding board bundle to entity: {:?}", entity);
        let mut ent_commands = commands.entity(entity);
        if maybe_spawner.is_none() {
            ent_commands.insert(VirusSpawner::default());
        }
        commands.entity(entity)
            .insert(BoardBundle::with_config(config))
            .insert(NeedsPill)
            .insert(NeedsSpawn);
    }
    state.set(GameState::Active);
}

pub(crate) fn check_if_finished(
    mut state: ResMut<NextState<GameState>>,
    query: Query<Entity, (Without<BoardFinished>, With<GameBoard>)>,
) {
    if query.is_empty() {
        info!("[check_if_finished] No unfinished boards left. Setting game state to finished.");
        state.set(GameState::Finished);
    }
}