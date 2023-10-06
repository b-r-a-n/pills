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
    query: Query<Entity, (Without<Finished>, With<GameBoard>)>,
) {
    if query.is_empty() {
        state.set(GameState::Finished);
    }
}

pub(crate) fn cleanup_game(
    mut commands: Commands,
    mut boards: Query<(Entity, &mut BoardConfig, &mut VirusSpawner, Option<&Finished>), With<GameBoard>>,
    pieces: Query<(Entity, &InBoard)>
) {
    for (board_ent, mut config, mut spawner, finished) in boards.iter_mut() {
        info!("[reset_game] Resetting board: {:?}", board_ent);
        if finished == Some(&Finished::Win) {
            spawner.advance();
            if config.drop_period > 0.2 {
                config.drop_period -= 0.1;
            }
        } else {
            spawner.reset();
        }
        commands.entity(board_ent)
            .remove::<(NeedsDrop, NeedsFall, NeedsSpawn, NeedsPill, NeedsResolve, NeedsSync, FallTimer, ResolveTimer, GameBoard, Move, Drop, Rotate, Finished)>();
        commands.entity(board_ent).despawn_descendants();
        for (ent, in_board_ent) in pieces.iter() {
            if in_board_ent.0 == board_ent {
                commands.entity(ent).despawn_recursive();
            }
        }
    }
}
