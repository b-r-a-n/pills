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
    query: Query<(Entity, &BoardConfig), Without<GameBoard>>,
) {
    for (entity, config) in query.iter() {
        info!("[start_game] Adding board bundle to entity: {:?}", entity);
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
    boards: Query<Entity, With<GameBoard>>,
    pieces: Query<(Entity, &InBoard)>
) {
    for board_ent in boards.iter() {
        info!("[reset_game] Resetting board: {:?}", board_ent);
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
