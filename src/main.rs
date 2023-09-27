use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle
};
use rand::{Rng, thread_rng};

use crate::game::board::*;

mod game;

/// Put systems here
/// 
fn setup_camera(
    mut commands: Commands
) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Resource)]
struct MeshHandles(Handle<Mesh>, Handle<Mesh>);

#[derive(Resource)]
struct MaterialHandles(Handle<ColorMaterial>, Handle<ColorMaterial>, Handle<ColorMaterial>);

fn setup_resources(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let virus_mesh_handle = meshes.add(shape::RegularPolygon::new(CELL_SIZE/2.0, 6).into());
    let pill_mesh_handle = meshes.add(shape::Circle::new(CELL_SIZE/2.0).into());
    commands.insert_resource(MeshHandles(virus_mesh_handle, pill_mesh_handle));

    let red_material = materials.add(Color::RED.into());
    let blue_material = materials.add(Color::BLUE.into());
    let yellow_material = materials.add(Color::YELLOW.into());
    commands.insert_resource(MaterialHandles(red_material, blue_material, yellow_material));

}

const CELL_SIZE: f32 = 32.0;
const BOARD_ROWS: u8 = 16;
const BOARD_COLS: u8 = 8;

fn spawn_board_background(
    mut commands: Commands,
) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(CELL_SIZE * BOARD_COLS as f32, CELL_SIZE * BOARD_ROWS as f32)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        },
        GameBoard,
    ));
}

fn spawn_viruses(
    mut commands: Commands,
    mut board: ResMut<Board<Entity>>,
    query: Query<Entity, With<GameBoard>>,
) {
    commands.entity(query.single()).with_children(|parent| {
        for row in 0..BOARD_ROWS-1 {
            // 3 is 100% for a virus, 300 is 1%
            let upper = u32::pow(3, row as u32 + 1);
            for column in 0..BOARD_COLS {
                let random_value = thread_rng().gen_range(0..upper);
                match random_value {
                    0..=2 => {
                        let color = match random_value {
                            0 => CellColor::RED,
                            1 => CellColor::BLUE,
                            2 => CellColor::YELLOW,
                            _ => unreachable!(),
                        };
                        let ent = parent.spawn((
                            Virus(color),
                            BoardPosition { row, column },
                        )).id();
                        board.set(row as usize, column as usize, Cell::Virus(ent, color));
                    },
                    _ => {}
                }
            }
        }
    });
}

fn rand_color() -> CellColor {
    match thread_rng().gen_range(0..3) {
        0 => CellColor::RED,
        1 => CellColor::BLUE,
        2 => CellColor::YELLOW,
        _ => unreachable!(),
    }
}

fn spawn_pill(
    mut commands: Commands,
    mut board: ResMut<Board<Entity>>,
    query: Query<Entity, With<GameBoard>>,
){
    commands.entity(query.single()).with_children(|parent| {
        board.set(
            (BOARD_ROWS-1) as usize, 
            (BOARD_COLS/2-1) as usize, 
            {
                let color = rand_color();
                let ent = parent.spawn((
                    Pill(color),
                    BoardPosition { row: BOARD_ROWS-1, column: BOARD_COLS/2-1 },
                    Controllable,
                )).id();
                Cell::Pill(ent, color, Some(Orientation::Right))
            }
        );
        board.set(
            (BOARD_ROWS-1) as usize,
            (BOARD_COLS/2) as usize, 
            {
                let color = rand_color();
                let ent = parent.spawn((
                    Pill(color),
                    BoardPosition { row: BOARD_ROWS-1, column: BOARD_COLS/2 },
                )).id();
                Cell::Pill(ent, color, Some(Orientation::Left))
            }
        );
    });
}

fn add_sprites(
    mut commands: Commands,
    mesh_handles: Res<MeshHandles>,
    material_handles: Res<MaterialHandles>,
    mut virus_query: Query<(Entity, &Virus, &BoardPosition), Added<Virus>>,
    mut pill_query: Query<(Entity, &Pill, &BoardPosition), Added<Pill>>,
) {
    for (entity, virus_type, board_position) in virus_query.iter_mut() {
        let (mesh, material) = match virus_type.0 {
            CellColor::RED => (mesh_handles.0.clone().into(), material_handles.0.clone()),
            CellColor::BLUE => (mesh_handles.0.clone().into(), material_handles.1.clone()),
            CellColor::YELLOW => (mesh_handles.0.clone().into(), material_handles.2.clone()),
            CellColor::GREEN => (mesh_handles.0.clone().into(), material_handles.0.clone()),
            CellColor::ORANGE => (mesh_handles.0.clone().into(), material_handles.0.clone()),
            CellColor::PURPLE => (mesh_handles.0.clone().into(), material_handles.0.clone()),
        };
        commands.entity(entity)
            .insert(MaterialMesh2dBundle { 
                mesh,
                material, 
                transform: Transform::from_translation(Vec3::new(board_position.column as f32 * CELL_SIZE, board_position.row as f32 * CELL_SIZE, 100.0)),
                ..default() 
        });
    }
    for (entity, pill_type, board_position) in pill_query.iter_mut() {
        let (mesh, material) = match pill_type.0 {
            CellColor::RED => (mesh_handles.1.clone().into(), material_handles.0.clone()),
            CellColor::BLUE => (mesh_handles.1.clone().into(), material_handles.1.clone()),
            CellColor::YELLOW => (mesh_handles.1.clone().into(), material_handles.2.clone()),
            CellColor::GREEN => (mesh_handles.1.clone().into(), material_handles.0.clone()),
            CellColor::ORANGE => (mesh_handles.1.clone().into(), material_handles.0.clone()),
            CellColor::PURPLE => (mesh_handles.1.clone().into(), material_handles.0.clone()),
        };
        commands.entity(entity)
            .insert(MaterialMesh2dBundle { 
                mesh,
                material, 
                transform: Transform::from_translation(Vec3::new(board_position.column as f32 * CELL_SIZE, board_position.row as f32 * CELL_SIZE, 100.0)),
                ..default() 
        });
    }
}

fn update_transforms(
    mut query: Query<(&BoardPosition, &mut Transform), Changed<BoardPosition>>,
) {
    for (board_position, mut transform) in query.iter_mut() {
        transform.translation.x = (board_position.column as f32 * CELL_SIZE) - (CELL_SIZE * BOARD_COLS as f32) / 2.0 + CELL_SIZE / 2.0;
        transform.translation.y = (board_position.row as f32 * CELL_SIZE) - (CELL_SIZE * BOARD_ROWS as f32) / 2.0 + CELL_SIZE / 2.0;
    }
}

fn start_game(
    mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
) {
    commands.insert_resource(Board::<Entity>::new(BOARD_ROWS as usize, BOARD_COLS as usize));
    state.set(GameState::PillDropping);
}

fn clear_matches(
    mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
    mut board: ResMut<Board<Entity>>,
) {
    let mut cells_cleared = false;
    let next_board = board.resolve(|lhs, rhs| lhs.color() == rhs.color());
    for i in 0..board.cells.len() {
        if next_board.cells[i] == Cell::Empty {
            match board.cells[i] {
                Cell::Virus(ent, _) => {
                    commands.entity(ent).despawn_recursive();
                    cells_cleared = true;
                }
                Cell::Pill(ent, _, _) => {
                    commands.entity(ent).despawn_recursive();
                    cells_cleared = true;
                }
                _ => {}
            }
        }
        board.cells[i] = next_board.cells[i];
    }
    if cells_cleared {
        state.set(GameState::PiecesFalling);
    } else {
        state.set(GameState::PillDropping);
    }
}

fn rotate_pill(
    mut control_query: Query<&mut BoardPosition, (With<Pill>, With<Controllable>)>,
    mut query: Query<&mut BoardPosition, (With<Pill>, Without<Controllable>)>,
    mut board: ResMut<Board<Entity>>,
    input: Res<Input<KeyCode>>,
) {
    let from = { 
        if let Ok(pos) = control_query.get_single() {
            (pos.row as usize, pos.column as usize) 
        } else { return }
    };
    let to = {
        if input.just_pressed(KeyCode::Z) {
            Orientation::Left
        } else if input.just_pressed(KeyCode::X) {
            Orientation::Right
        } else {
            return
        }
    };

    if board.rotate_pill(from, to) {
        let mut pos = control_query.single_mut();
        pos.row = from.0 as u8;
        pos.column = from.1 as u8;

        // This is how we keep the entities in sync with the board.
        // This should only update at most 1 entity (which is the connected cell if it exists) 
        for i in 0..board.cells.len() {
            match board.cells[i] {
                Cell::Pill(ent, _, _) => {
                    if let Ok(mut other_pos) = query.get_mut(ent) {
                        other_pos.row = (i / board.cols) as u8;
                        other_pos.column = (i % board.cols) as u8;
                    }
                }
                _ => {}
            }
        }
    }
}

fn move_pill(
    mut control_query: Query<&mut BoardPosition, (With<Pill>, With<Controllable>)>,
    mut query: Query<&mut BoardPosition, (With<Pill>, Without<Controllable>)>,
    mut board: ResMut<Board<Entity>>,
    input: Res<Input<KeyCode>>,
) {
    let from = { 
        if let Ok(pos) = control_query.get_single() {
            (pos.row as usize, pos.column as usize) 
        } else { return }
    };
    let to = { 
        let mut pos = from;
        if input.just_pressed(KeyCode::Left) {
            if pos.1 > 0 {
                pos.1 -= 1;
            }
        }
        if input.just_pressed(KeyCode::Right) {
            if pos.1 < board.cols - 1 {
                pos.1 += 1;
            }
        }
        pos
    };
    if from == to { return }
    if board.move_pill(from, to) {
        let mut pos = control_query.single_mut();
        pos.row = to.0 as u8;
        pos.column = to.1 as u8;

        // This is how we keep the entities in sync with the board.
        // This should only update at most 1 entity (which is the connected cell if it exists) 
        for i in 0..board.cells.len() {
            match board.cells[i] {
                Cell::Pill(ent, _, _) => {
                    if let Ok(mut other_pos) = query.get_mut(ent) {
                        other_pos.row = (i / board.cols) as u8;
                        other_pos.column = (i % board.cols) as u8;
                    }
                }
                _ => {}
            }
        }
    }
}

fn drop_pieces(
    mut board: ResMut<Board<Entity>>,
    mut query: Query<&mut BoardPosition, With<Pill>>,
    mut state: ResMut<NextState<GameState>>,
) {

    let next_board = board.next();
    let mut piece_moved = false;
    for i in 0..next_board.cells.len() {
        match next_board.cells[i] {
            Cell::Pill(ent, _, _) => {
                if let Ok(mut pos) = query.get_mut(ent) {
                    if pos.row != (i / board.cols) as u8 {
                        pos.row = (i / board.cols) as u8;
                        piece_moved = true;
                    }
                }
            }
            _ => {}
        }
    }
    if piece_moved {
        *board = next_board;
    } else {
        state.set(GameState::Resolving);
    }
}

fn drop_pill(
    mut commands: Commands,
    mut board: ResMut<Board<Entity>>,
    mut query: Query<&mut BoardPosition, With<Pill>>,
    mut state: ResMut<NextState<GameState>>,
    control_query: Query<Entity, With<Controllable>>,
) {
    
    let (entity, row, column) = {
        let pivot_ent = control_query.single();
        let pivot_pos = query.get(pivot_ent).unwrap();
        (pivot_ent, pivot_pos.row as usize, pivot_pos.column as usize)
    };
    if row > 0 && board.move_pill((row, column), (row-1, column)) {
        // Fixup the entities
        if let (cell, Some((paired_cell, _, _ ))) = board.get_paired(row-1, column) {
            cell.get().map(|ent| query.get_mut(ent).unwrap().row -= 1);
            paired_cell.get().map(|ent| query.get_mut(ent).unwrap().row -= 1);
        } else {
            warn!("No paired cell found for pill at ({}, {})", row, column);
        }
    } else {
        // TODO: This is where we can check for game over
        commands.entity(entity).remove::<Controllable>();
        state.set(GameState::Resolving);
    }
}

fn reset_fall_time(
    mut commands: Commands,
) {
    commands.insert_resource(FixedTime::new_from_secs(0.3));
}

fn reset_drop_time(
    mut commands: Commands,
) {
    commands.insert_resource(FixedTime::new_from_secs(0.8));
}

fn adjust_drop_time(
    mut time: ResMut<FixedTime>,
    input: Res<Input<KeyCode>>,
) {
    if input.pressed(KeyCode::Down) {
        time.tick(core::time::Duration::from_secs_f32(0.1));
    }
}

fn check_for_game_over(
    mut state: ResMut<NextState<GameState>>,
    board: Res<Board<Entity>>,
) {
    if board.virus_count() < 1 {
        state.set(GameState::Finished);
    }
}

fn cleanup_game(
    mut commands: Commands,
    query: Query<Entity, With<GameBoard>>,
) {
    for ent in query.iter() {
        commands.entity(ent).despawn_descendants();
    }
}

fn reset_game(
    mut state: ResMut<NextState<GameState>>,
    input: Res<Input<KeyCode>>,
) {
    if input.pressed(KeyCode::Return) {
        state.set(GameState::Starting);
    }
}

fn startup_finished(
    mut state: ResMut<NextState<GameState>>
) {
    state.set(GameState::Starting);
}

/// Put components here
/// 
#[derive(Component)]
struct BoardPosition {
    row: u8,
    column: u8,
}

#[derive(Component)]
struct Virus(CellColor);

#[derive(Component)]
struct Pill(CellColor);

#[derive(Component)]
struct Controllable;

#[derive(Component)]
struct GameBoard;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
enum GameState {
    #[default]
    NotStarted,
    Starting,
    PillDropping,
    Resolving,
    PiecesFalling,
    Finished,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state::<GameState>()
        .add_systems(Startup, (setup_resources, setup_camera, spawn_board_background))
        .add_systems(PostStartup, startup_finished)
        .add_systems(OnEnter(GameState::Starting), start_game)
        .add_systems(OnExit(GameState::Starting), spawn_viruses)
        .add_systems(OnEnter(GameState::PillDropping), (reset_drop_time, spawn_pill))
        .add_systems(OnEnter(GameState::PiecesFalling), reset_fall_time)
        .add_systems(OnEnter(GameState::Resolving), clear_matches)
        .add_systems(OnExit(GameState::Resolving), check_for_game_over)
        .add_systems(OnExit(GameState::Finished), cleanup_game)
        .add_systems(
            Update, 
            (
                add_sprites, 
                (adjust_drop_time, move_pill, rotate_pill).run_if(in_state(GameState::PillDropping)),
                reset_game.run_if(in_state(GameState::Finished)),
                bevy::window::close_on_esc))
        .add_systems(
            FixedUpdate, 
            (
                drop_pieces.run_if(in_state(GameState::PiecesFalling)),
                drop_pill.run_if(in_state(GameState::PillDropping))))
        .insert_resource(FixedTime::new_from_secs(1.))
        .add_systems(PostUpdate, update_transforms.before(bevy::transform::TransformSystem::TransformPropagate))
        .run();
}
