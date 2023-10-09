use bevy::prelude::*;
use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use pills_game_board::*;
use rand::{thread_rng, Rng};
use rand::rngs::ThreadRng;

pub use pills_pieces::*;
pub use game_state::*;

mod game_state;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state::<GameState>()
            .add_event::<PillEvent>()
            .add_event::<ClearEvent>()
            .add_systems(OnEnter(GameState::Starting), start_game)
            .add_systems(OnExit(GameState::Starting), (spawn_viruses, spawn_pill))
            .add_systems(
                Update, 
                (
                    add_pill_to_board, 
                    spawn_pill, 
                    move_pill, 
                    drop_pill,
                    rotate_pill,
                    drop_pieces,
                    resolve_timer,
                    clear_matches,
                    clear_cleared,
                    despawn,
                    sync_with_board,
                )
                    .run_if(in_state(GameState::Active))
            )
            .add_systems(PostUpdate, check_board_state.run_if(in_state(GameState::Active)))
        ;
    }
}

#[derive(Bundle)]
pub struct BoardBundle {
    board: GameBoard,
    fall_timer: FallTimer,
}

impl BoardBundle {
    pub fn with_config(config: &BoardConfig) -> Self {
        let (rows, cols) = config.board_size;
        Self {
            board: GameBoard(Board::new(rows, cols)),
            fall_timer: FallTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
        }
    }
}

pub(crate) type SpawnPolicy = fn(&mut VirusSpawner, &mut ThreadRng, u8, u8) -> Option<Virus>;

#[derive(Component)]
pub(crate) struct VirusSpawner {
    max_viruses: usize,
    spawned_count: usize,
    spawn_policy: SpawnPolicy,
}

impl VirusSpawner {
    fn spawn(&mut self, rng: &mut ThreadRng, row: u8, col: u8) -> Option<Virus> {
        if self.max_viruses > self.spawned_count {
            let result = (self.spawn_policy)(self, rng, row, col);
            if result.is_some() {
                self.spawned_count += 1;
            }
            result
        } else {
            None
        }
    }

    fn advance(&mut self) {
        self.spawned_count = 0;
        self.max_viruses *= 2;
    }
}

impl Default for VirusSpawner {
    fn default() -> Self {
        Self {
            max_viruses: 1,
            spawned_count: 0,
            spawn_policy: |_, rng, _, _| {
                match rng.gen_range(0..4) {
                    0 => Some(Virus(CellColor::RED)),
                    1 => Some(Virus(CellColor::BLUE)),
                    2 => Some(Virus(CellColor::YELLOW)),
                    _ => None,
                }
            },
        }
    }
}

#[derive(Component)]
pub struct DespawnIn(pub Timer);

fn despawn(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DespawnIn)>,
    time: Res<Time>
) {

    for (entity, mut timer) in query.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }

}

fn spawn_viruses(
    mut commands: Commands,
    mut query: Query<(Entity, &mut VirusSpawner, &mut GameBoard)>,
) {
    for (entity, mut spawner, mut board) in query.iter_mut() {
        commands.entity(entity).with_children(|builder|{
            for row in 0..(board.rows-1) as u8 {
                if spawner.max_viruses < 1 {
                    break;
                }
                for col in 0..board.cols as u8 {
                    let mut rng = thread_rng();
                    if let Some(virus) = spawner.spawn(&mut rng, row, col) {
                        let ent = builder.spawn((
                            virus, 
                            BoardPosition { row, column: col },
                            InBoard(entity),
                        )).id();
                        board.set(row as usize, col as usize, Cell::Virus(ent, virus.0));
                    }
                }
            }
        });
        spawner.advance();
    }
}

fn spawn_pill(
    mut commands: Commands,
    query: Query<Entity, (With<GameBoard>, With<NeedsSpawn>)>
) {
    for entity in query.iter() {
        info!("[spawn_pill] Spawning pill for board: {:?}", entity);
        commands.spawn_batch([
            (Pill(rand_color()), NextPill(0), InBoard(entity)),
            (Pill(rand_color()), NextPill(1), InBoard(entity)),
        ]);
        commands.entity(entity).remove::<NeedsSpawn>();
    }
}

fn add_pill_to_board(
    mut commands: Commands,
    mut boards: Query<(Entity, &mut GameBoard), (With<NeedsPill>, Without<NeedsSpawn>, Without<NeedsDrop>, Without<NeedsSync>)>,
    next_pieces: Query<(Entity, &Pill, &NextPill, &InBoard)>
) {
    // For each pill marked with NextPill
    for (piece_ent, pill, piece_index, board_ent) in next_pieces.iter() {
        if let Ok((board_ent, mut board)) = boards.get_mut(**board_ent) {
            let (row, col) = (board.rows-1, board.cols/2-1);
            let col = col + piece_index.0 as usize;
            let orientation = if piece_index.0 == 0 { 
                Some(Orientation::Right) 
            } else { 
                Some(Orientation::Left) 
            };
            if board.get(row, col) != Cell::Empty {
                info!("[add_pill_to_board] Pill cannot be added to board {:?} at {}, {}", board_ent, row, col);
                commands.entity(board_ent).insert(BoardFinished::Loss);
                continue;
            }
            board.set(row, col, Cell::Pill(piece_ent, pill.0, orientation));
            commands.entity(piece_ent)
                .remove::<NextPill>()
                .insert(BoardPosition { row: row as u8, column: col as u8 })
                .insert(InBoard(board_ent))
                .set_parent(board_ent);
            if piece_index.0 == 1 {
                commands.entity(piece_ent)
                    .insert(PivotPiece);
            }
            commands.entity(board_ent)
                .remove::<NeedsPill>()
                .insert(NeedsDrop)
                .insert(NeedsSpawn);
        }
    }
}

fn resolve_timer(
    mut commands: Commands,
    mut timer: Query<(Entity, &mut ResolveTimer), With<NeedsResolve>>,
    time: Res<Time>,
) {
    for (entity, mut timer) in timer.iter_mut() {
        if timer.tick(time.delta()).just_finished() { 
            commands.entity(entity)
                .insert(NeedsFall)
                .remove::<ResolveTimer>()
                .remove::<NeedsResolve>();
        }
    }
}

fn clear_cleared(
    mut commands: Commands,
    cleared_query: Query<(Entity, &Parent), With<ClearedCell>>,
    resolve_board_query: Query<Entity, Without<NeedsResolve>>,
) {
    for (cell_entity, board_entity) in cleared_query.iter() {
        if resolve_board_query.get(board_entity.get()).is_ok() {
            commands.entity(cell_entity).despawn_recursive();
        }
    }
}

fn check_board_state(
    mut commands: Commands,
    boards: Query<(Entity, &GameBoard), (Changed<GameBoard>, Without<BoardFinished>)>,
) {
    for (entity, board) in boards.iter() {
        if board.virus_count() < 1 {
            info!("[check_board_state] Board {:?} has no viruses. Marking as finished", entity);
            commands.entity(entity).insert(BoardFinished::Win);
        }
    }
}

fn clear_matches(
    mut commands: Commands,
    mut board_query: Query<(Entity, &mut GameBoard), (With<NeedsResolve>, Without<ResolveTimer>)>,
    mut events: EventWriter<ClearEvent>,
) {
    for (entity, mut board) in board_query.iter_mut() {
        let next_board = board.resolve(|l, r| l.color() == r.color());
        if next_board == **board {
            commands.entity(entity)
                .insert(NeedsPill)
                .remove::<NeedsResolve>();
            return;
        }
        commands.entity(entity)
            .insert(ResolveTimer(Timer::from_seconds(0.4, TimerMode::Once)));
        for row in 0..board.rows {
            for col in 0..board.cols {
                if next_board.get(row, col) == Cell::Empty {
                    match board.get(row, col) {
                        Cell::Pill(ent, color, _) => {
                            commands.entity(ent).despawn_recursive();
                            commands.spawn((
                                BoardPosition { row: row as u8, column: col as u8 },
                                ClearedCell {color, was_virus: false},
                                InBoard(entity),
                            )).set_parent(entity);
                        },
                        Cell::Virus(ent, color) => {
                            commands.entity(ent).despawn_recursive();
                            commands.spawn((
                                BoardPosition { row: row as u8, column: col as u8 },
                                ClearedCell {color, was_virus: true},
                                InBoard(entity)
                            )).set_parent(entity);
                        },
                        _ => {},
                    }
                }
            }
        }
        events.send(ClearEvent(entity));
        **board = next_board;
    }
}

fn move_pill(
    mut commands: Commands,
    mut board_query: Query<&mut GameBoard, Without<NeedsSync>>,
    mut events: EventWriter<PillEvent>,
    query: Query<(Entity, &BoardPosition, &InBoard, &Move), With<PivotPiece>>,
) {
    for (entity, pos, board_entity, movement) in query.iter() {
        if let Ok(mut board) = board_query.get_mut(**board_entity) {
            let from = (pos.row as usize, pos.column as usize);
            let mut to = (pos.row as usize, pos.column as usize);
            match movement {
                Move::Left => {
                    if pos.column > 0 {
                        to.1 -= 1;
                    }
                },
                Move::Right => {
                    if (pos.column as usize) < board.cols - 1 {
                        to.1 += 1;
                    }
                },
            }
            if board.move_pill(from, to) {
                commands.entity(entity).remove::<Move>();
                commands.entity(**board_entity).insert(NeedsSync);
                events.send(PillEvent::PillMoved(**board_entity, entity));
            }
        }
    }
}

fn rotate_pill(
    mut commands: Commands,
    mut board_query: Query<&mut GameBoard, Without<NeedsSync>>,
    mut events: EventWriter<PillEvent>,
    query: Query<(Entity, &BoardPosition, &InBoard, &Rotate), With<PivotPiece>>,
) {
    for (entity, pos, board_entity, rotation) in query.iter() {
        if let Ok(mut board) = board_query.get_mut(**board_entity) {
            let from = (pos.row as usize, pos.column as usize);
            let to = match rotation { Rotate::Left => Orientation::Left, Rotate::Right => Orientation::Right };
            if board.rotate_pill(from, to) { 
                commands.entity(entity).remove::<Rotate>();
                commands.entity(**board_entity).insert(NeedsSync);
                events.send(PillEvent::PillRotated(**board_entity, entity))
            } 
        }
    }
}

fn drop_pill(
    mut commands: Commands,
    query: Query<(Entity, &BoardPosition, &InBoard), (With<PivotPiece>, With<Drop>)>,
    mut board_query: Query<&mut GameBoard, Without<NeedsSync>>,
) {
    for (entity, pos, board_entity) in query.iter() {
        if let Ok(mut board) = board_query.get_mut(**board_entity) {
            if pos.row > 0 && board.move_pill((pos.row as usize, pos.column as usize), (pos.row as usize - 1, pos.column as usize)) {
                commands.entity(entity).remove::<Drop>();
                commands.entity(**board_entity).insert(NeedsSync);
            } else {
                commands.entity(entity).remove::<PivotPiece>();
                commands.entity(**board_entity)
                    .remove::<NeedsDrop>()
                    .insert(NeedsResolve);
            }
        }
    }
}

fn drop_pieces(
    mut commands: Commands,
    mut board: Query<(Entity, &mut GameBoard, &mut FallTimer), (With<NeedsFall>, Without<NeedsSync>)>,
    time: Res<Time>,
) {
    for (entity, mut board, mut timer) in board.iter_mut() {
        if timer.tick(time.delta()).just_finished() {
            let next_board = board.next();
            if next_board != **board {
                **board = next_board;
                commands.entity(entity).insert(NeedsSync);
            } else {
                commands.entity(entity)
                    .insert(NeedsResolve)
                    .remove::<NeedsFall>();
            }
        }
    }
}

fn sync_with_board(
    mut commands: Commands,
    board_query: Query<(Entity, &GameBoard), With<NeedsSync>>,
    mut position_query: Query<&mut BoardPosition>,
) {
    for (board_entity, board) in board_query.iter() {
        for row in 0..board.rows {
            for col in 0..board.cols {
                if let Cell::Pill(pill_ent, _, maybe_orientation) = board.get(row, col) {
                    if let Ok(mut pos) = position_query.get_mut(pill_ent) {
                        // TODO: This is to handle the case where the pill is in the middle of a rotation
                        // Since there is no orientation component, the renderer just uses the board data to get orientation
                        // However, only one piece moves during a rotation, so the other piece will be out of sync w.r.t its rotation
                        if maybe_orientation.is_some() {
                            pos.row = row as u8;
                            pos.column = col as u8;
                        } else {
                            pos.set_if_neq(BoardPosition { row: row as u8, column: col as u8 });
                        }
                    }
                }
            }
        }
        commands.entity(board_entity).remove::<NeedsSync>();
    }
}

#[derive(Component, Debug)]
pub enum Move {
    Left,
    Right,
}

#[derive(Component, Debug)]
pub struct Drop;

#[derive(Component, Debug)]
pub enum Rotate {
    Left,
    Right
}

#[derive(Component, Debug)]
struct NeedsResolve;

#[derive(Component, Debug)]
struct NeedsSync;

#[derive(Component)]
pub struct PivotPiece;

#[derive(Component, Debug)]
struct NeedsPill;

#[derive(Component, Debug)]
struct NeedsSpawn;

#[derive(Component, Debug)]
struct NeedsDrop;

#[derive(Component, Debug)]
struct NeedsFall;

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub enum BoardFinished {
    Win,
    Loss,
}

pub struct CorePluginGroup;

impl PluginGroup for CorePluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(GamePlugin)
    }
}

fn rand_color() -> CellColor {
    match thread_rng().gen_range(0..3) {
        0 => CellColor::RED,
        1 => CellColor::BLUE,
        2 => CellColor::YELLOW,
        _ => unreachable!(),
    }
}

#[derive(Component)]
pub struct BoardConfig {
    pub board_size: (usize, usize),
    pub max_viruses: usize,
    pub drop_period: f32,
    pub fall_period: f32,
}

impl Default for BoardConfig {
    fn default() -> Self {
        Self {
            board_size: (16, 8),
            max_viruses: 1,
            drop_period: 0.8,
            fall_period: 0.2,
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct FallTimer(pub Timer);

#[derive(Component, Deref, DerefMut)]
struct ResolveTimer(pub Timer);

#[derive(Event, Debug)]
pub enum PillEvent {
    PillMoved(Entity, Entity),
    PillRotated(Entity, Entity),
}

#[derive(Event, Debug)]
pub struct ClearEvent(Entity);