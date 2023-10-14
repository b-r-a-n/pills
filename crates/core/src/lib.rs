use bevy::prelude::*;
use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use pills_game_board::*;
use rand::{thread_rng, Rng};
use rand::rngs::ThreadRng;

pub use game_state::*;
pub use events::*;

mod game_state;
mod events;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BoardEvent>()
            .add_state::<GameState>()
            .add_systems(
                OnEnter(GameState::Starting), 
                start_game)
            .add_systems(
                OnTransition { from: GameState::Starting, to: GameState::Active}, 
                (spawn_viruses, spawn_pill))
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
                    sync_with_board)
                        .run_if(in_state(GameState::Active)))
            .add_systems(PostUpdate, check_board_state.run_if(in_state(GameState::Active)))
        ;
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct BoardPlayer(pub Entity);

#[derive(Component)]
pub struct BoardInfoContainer(pub Entity);

#[derive(Component, Debug, PartialEq)]
pub struct BoardPosition {
    pub row: u8,
    pub column: u8,
}

#[derive(Component, Deref, DerefMut)]
pub struct GameBoard(pub Board<Entity>);

#[derive(Component)]
pub struct NextPill(pub u8);

#[derive(Component, Deref, DerefMut)]
pub struct InBoard(pub Entity);

#[derive(Component)]
pub struct Stacked(pub usize);

#[derive(Component)]
pub struct RemoveStack(pub usize);

#[derive(Bundle)]
pub struct BoardBundle {
    board: GameBoard,
    fall_timer: FallTimer,
    virus_spawner: VirusSpawner,
}

impl BoardBundle {
    pub fn with_config(config: &BoardConfig) -> Self {
        let (rows, cols) = config.board_size;
        Self {
            board: GameBoard(Board::new(rows, cols)),
            fall_timer: FallTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
            virus_spawner: VirusSpawner { max_viruses: config.max_viruses, ..default()}
        }
    }
}

#[derive(Clone, Copy, Component, Debug)]
pub struct Virus(pub CellColor);

#[derive(Clone, Copy, Component, Debug)]
pub struct Pill(pub CellColor);

#[derive(Clone, Copy, Component)]
pub struct ClearedCell {
    pub color: CellColor,
    pub was_virus: bool,
}

pub type SpawnPolicy = fn(&mut VirusSpawner, &mut ThreadRng, u8, u8) -> Option<Virus>;

#[derive(Component)]
pub struct VirusSpawner {
    pub max_viruses: usize,
    pub spawn_policy: SpawnPolicy,
    spawned_count: usize,
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
    mut events: EventWriter<BoardEvent>,
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
                commands.entity(board_ent).insert(BoardFinished::Loss);
                continue;
            }
            board.set(row, col, Cell::Pill(piece_ent, pill.0, orientation));
            events.send(BoardEvent::pill_added(board_ent, piece_ent, *pill));
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
            commands.entity(entity).insert(BoardFinished::Win);
        }
    }
}

fn clear_matches(
    mut commands: Commands,
    mut board_query: Query<(Entity, &mut GameBoard), (With<NeedsResolve>, Without<ResolveTimer>)>,
    mut stacks: Query<&mut Stacked>,
    mut remove_stacks: Query<&mut RemoveStack>,
    mut events: EventWriter<BoardEvent>,
) {
    for (board_id, mut board) in board_query.iter_mut() {
        let (mut next_board, mask) = board.resolve(|l, r| l.color() == r.color());
        if next_board == **board {
            commands.entity(board_id)
                .insert(NeedsPill)
                .remove::<NeedsResolve>();
            return;
        }

        // Process the mask to check the masked cells for remove_stack components
        // Mask : u8 that identifies match groups
        // MaskLookup : map from the mask value to a bool indicating if the match group has a remove stack component
        let mut mask_lookup: Vec<Option<usize>> = vec![None; mask.len()];
        for (index, cell) in board.cells.iter().enumerate() {
            let mask_index = mask[index] as usize;
            if mask_index > 0 {
                if let Cell::Pill(id, _, _) = cell {
                    if let Ok(remove_stack) = remove_stacks.get(*id) {
                        let stack_val = mask_lookup[mask_index].unwrap_or(0);
                        if remove_stack.0 > stack_val {
                            mask_lookup[mask_index] = Some(remove_stack.0);
                        }
                    }
                }
            }
        }

        commands.entity(board_id)
            .insert(ResolveTimer(Timer::from_seconds(0.3, TimerMode::Once)));
        let mut amount = 0;
        for row in 0..board.rows {
            for col in 0..board.cols {
                if next_board.get(row, col) == Cell::Empty {
                    let mut cell_id: Option<Entity> = None;
                    let mut was_virus = false;
                    let mut color = CellColor::RED;
                    match board.get(row, col) {
                        Cell::Pill(ent, pill_color, _) => {
                            cell_id = Some(ent);
                            color = pill_color;
                        },
                        Cell::Virus(ent, virus_color) => {
                            cell_id = Some(ent);
                            color = virus_color;
                            was_virus = true;
                        },
                        _ => {},
                    }
                    if let Some(cell_id) = cell_id {
                        let should_clear = if let Ok(mut stacks) = stacks.get_mut(cell_id) {
                            let cell_index = board.get_index(row, col);
                            let mask_index = mask[cell_index] as usize;
                            let stacks_removed = mask_lookup[mask_index].unwrap_or(1);
                            if stacks.0 > 0 {
                                let removed_amount = std::cmp::min(stacks_removed, stacks.0);
                                stacks.0 -= removed_amount;
                            }
                            if stacks.0 == 1 {
                                commands.entity(cell_id).remove::<Stacked>();
                            }
                            stacks.0 < 1
                        } else {
                            true
                        };
                        if should_clear {
                            commands.entity(cell_id).despawn_recursive();
                            commands.spawn((
                                BoardPosition { row: row as u8, column: col as u8 },
                                ClearedCell { color, was_virus},
                                InBoard(board_id),
                            )).set_parent(board_id);
                            if was_virus {
                                events.send(BoardEvent::virus_removed(board_id, cell_id, Virus(color), row as u8, col as u8));
                            }
                            amount += 1;
                        } else {
                            let cell = board.get(row, col);
                            next_board.set(row, col, cell);
                        }
                    }
                }
            }
        }
        events.send(BoardEvent::cells_cleared(board_id, amount));
        **board = next_board;
    }
}

fn move_pill(
    mut commands: Commands,
    mut board_query: Query<&mut GameBoard, Without<NeedsSync>>,
    mut events: EventWriter<BoardEvent>,
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
            if from != to && board.move_pill(from, to) {
                commands.entity(**board_entity).insert(NeedsSync);
                events.send(BoardEvent::pill_moved(**board_entity, entity, *movement));
            }
            commands.entity(entity).remove::<Move>();
        }
    }
}

fn rotate_pill(
    mut commands: Commands,
    mut board_query: Query<&mut GameBoard, Without<NeedsSync>>,
    mut events: EventWriter<BoardEvent>,
    query: Query<(Entity, &BoardPosition, &InBoard, &Rotate), With<PivotPiece>>,
) {
    for (entity, pos, board_entity, rotation) in query.iter() {
        if let Ok(mut board) = board_query.get_mut(**board_entity) {
            let from = (pos.row as usize, pos.column as usize);
            let to = match rotation { Rotate::Left => Orientation::Left, Rotate::Right => Orientation::Right };
            if board.rotate_pill(from, to) { 
                commands.entity(entity).remove::<Rotate>();
                commands.entity(**board_entity).insert(NeedsSync);
                events.send(BoardEvent::pill_moved(**board_entity, entity, *rotation))
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

#[derive(Clone, Copy, Component, Debug)]
pub enum Move {
    Left,
    Right,
}

#[derive(Component, Debug)]
pub struct Drop;

#[derive(Clone, Copy, Component, Debug)]
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
