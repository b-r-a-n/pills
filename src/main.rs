use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle
};
use rand::{Rng, thread_rng};

use crate::game::grid::*;

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
const GRID_ROWS: u8 = 16;
const GRID_COLS: u8 = 8;

fn spawn_board(
    mut commands: Commands,
) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(CELL_SIZE * GRID_COLS as f32, CELL_SIZE * GRID_ROWS as f32)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        },
        Board,
    ));
}

fn spawn_viruses(
    mut commands: Commands,
    mut grid: ResMut<Grid<Entity>>,
    query: Query<Entity, With<Board>>,
) {
    commands.entity(query.single()).with_children(|parent| {
        for row in 0..GRID_ROWS-1 {
            for column in 0..GRID_COLS {
                let random_value = thread_rng().gen_range(0..100);
                match random_value {
                    0..=2 => {
                        let color = match random_value {
                            0 => GridColor::RED,
                            1 => GridColor::BLUE,
                            2 => GridColor::YELLOW,
                            _ => unreachable!(),
                        };
                        let ent = parent.spawn((
                            Virus(color),
                            GridPosition { row, column },
                        )).id();
                        grid.set(row as usize, column as usize, Cell::Virus(ent, color));
                    },
                    _ => {}
                }
            }
        }
    });
}

fn rand_color() -> GridColor {
    match thread_rng().gen_range(0..3) {
        0 => GridColor::RED,
        1 => GridColor::BLUE,
        2 => GridColor::YELLOW,
        _ => unreachable!(),
    }
}

fn spawn_pill(
    mut commands: Commands,
    mut grid: ResMut<Grid<Entity>>,
    query: Query<Entity, With<Board>>,
){
    commands.entity(query.single()).with_children(|parent| {
        grid.set(
            (GRID_ROWS-1) as usize, 
            (GRID_COLS/2-1) as usize, 
            {
                let color = rand_color();
                let ent = parent.spawn((
                    Pill(color),
                    GridPosition { row: GRID_ROWS-1, column: GRID_COLS/2-1 },
                    Controllable,
                )).id();
                Cell::Pill(ent, color, Some(Orientation::Right))
            }
        );
        grid.set(
            (GRID_ROWS-1) as usize,
            (GRID_COLS/2) as usize, 
            {
                let color = rand_color();
                let ent = parent.spawn((
                    Pill(color),
                    GridPosition { row: GRID_ROWS-1, column: GRID_COLS/2 },
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
    mut virus_query: Query<(Entity, &Virus, &GridPosition), Added<Virus>>,
    mut pill_query: Query<(Entity, &Pill, &GridPosition), Added<Pill>>,
) {
    for (entity, virus_type, grid_position) in virus_query.iter_mut() {
        let (mesh, material) = match virus_type.0 {
            GridColor::RED => (mesh_handles.0.clone().into(), material_handles.0.clone()),
            GridColor::BLUE => (mesh_handles.0.clone().into(), material_handles.1.clone()),
            GridColor::YELLOW => (mesh_handles.0.clone().into(), material_handles.2.clone()),
            GridColor::GREEN => (mesh_handles.0.clone().into(), material_handles.0.clone()),
            GridColor::ORANGE => (mesh_handles.0.clone().into(), material_handles.0.clone()),
            GridColor::PURPLE => (mesh_handles.0.clone().into(), material_handles.0.clone()),
        };
        commands.entity(entity)
            .insert(MaterialMesh2dBundle { 
                mesh,
                material, 
                transform: Transform::from_translation(Vec3::new(grid_position.column as f32 * CELL_SIZE, grid_position.row as f32 * CELL_SIZE, 100.0)),
                ..default() 
        });
    }
    for (entity, pill_type, grid_position) in pill_query.iter_mut() {
        let (mesh, material) = match pill_type.0 {
            GridColor::RED => (mesh_handles.1.clone().into(), material_handles.0.clone()),
            GridColor::BLUE => (mesh_handles.1.clone().into(), material_handles.1.clone()),
            GridColor::YELLOW => (mesh_handles.1.clone().into(), material_handles.2.clone()),
            GridColor::GREEN => (mesh_handles.1.clone().into(), material_handles.0.clone()),
            GridColor::ORANGE => (mesh_handles.1.clone().into(), material_handles.0.clone()),
            GridColor::PURPLE => (mesh_handles.1.clone().into(), material_handles.0.clone()),
        };
        commands.entity(entity)
            .insert(MaterialMesh2dBundle { 
                mesh,
                material, 
                transform: Transform::from_translation(Vec3::new(grid_position.column as f32 * CELL_SIZE, grid_position.row as f32 * CELL_SIZE, 100.0)),
                ..default() 
        });
    }
}

fn update_transforms(
    mut query: Query<(&GridPosition, &mut Transform), Changed<GridPosition>>,
) {
    for (grid_position, mut transform) in query.iter_mut() {
        transform.translation.x = (grid_position.column as f32 * CELL_SIZE) - (CELL_SIZE * GRID_COLS as f32) / 2.0 + CELL_SIZE / 2.0;
        transform.translation.y = (grid_position.row as f32 * CELL_SIZE) - (CELL_SIZE * GRID_ROWS as f32) / 2.0 + CELL_SIZE / 2.0;
    }
}

fn start_game(
    mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
) {
    commands.insert_resource(Grid::<Entity>::new(GRID_ROWS as usize, GRID_COLS as usize));
    state.set(GameState::PillDropping);
}

fn clear_matches(
    mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
    mut grid: ResMut<Grid<Entity>>,
) {
    let mut cells_cleared = false;
    let next_grid = grid.resolve(|lhs, rhs| lhs.color() == rhs.color());
    for i in 0..grid.cells.len() {
        if next_grid.cells[i] == Cell::Empty {
            match grid.cells[i] {
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
        grid.cells[i] = next_grid.cells[i];
    }
    if cells_cleared {
        state.set(GameState::PillsFalling);
    } else {
        state.set(GameState::PillDropping);
    }
}

fn rotate_pill(
    mut control_query: Query<&mut GridPosition, (With<Pill>, With<Controllable>)>,
    mut query: Query<&mut GridPosition, (With<Pill>, Without<Controllable>)>,
    mut grid: ResMut<Grid<Entity>>,
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

    if grid.rotate_pill(from, to) {
        let mut pos = control_query.single_mut();
        pos.row = from.0 as u8;
        pos.column = from.1 as u8;

        // This is how we keep the entities in sync with the grid.
        // This should only update at most 1 entity (which is the connected cell if it exists) 
        for i in 0..grid.cells.len() {
            match grid.cells[i] {
                Cell::Pill(ent, _, _) => {
                    if let Ok(mut other_pos) = query.get_mut(ent) {
                        other_pos.row = (i / grid.cols) as u8;
                        other_pos.column = (i % grid.cols) as u8;
                    }
                }
                _ => {}
            }
        }
    }
}

fn move_pill(
    mut control_query: Query<&mut GridPosition, (With<Pill>, With<Controllable>)>,
    mut query: Query<&mut GridPosition, (With<Pill>, Without<Controllable>)>,
    mut grid: ResMut<Grid<Entity>>,
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
            if pos.1 < grid.cols - 1 {
                pos.1 += 1;
            }
        }
        pos
    };
    if from == to { return }
    if grid.move_pill(from, to) {
        let mut pos = control_query.single_mut();
        pos.row = to.0 as u8;
        pos.column = to.1 as u8;

        // This is how we keep the entities in sync with the grid.
        // This should only update at most 1 entity (which is the connected cell if it exists) 
        for i in 0..grid.cells.len() {
            match grid.cells[i] {
                Cell::Pill(ent, _, _) => {
                    if let Ok(mut other_pos) = query.get_mut(ent) {
                        other_pos.row = (i / grid.cols) as u8;
                        other_pos.column = (i % grid.cols) as u8;
                    }
                }
                _ => {}
            }
        }
    }
}

fn move_pills(
    mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
    mut query: Query<(&mut Pill, &mut GridPosition)>,
    control_query: Query<Entity, (With<Pill>, With<Controllable>)>,
    grid: Res<Grid<Entity>>,
) {
    let next_grid = grid.next();
    let mut pills_moved = false;
    for cell in grid.cells.iter() {
        match cell {
            Cell::Pill(ent, _, _) => {

                // The pill's pos before any movement
                let maybe_pill = query.get_mut(*ent);
                if maybe_pill.is_err() {
                    // TODO: How do we get here?
                    // I suspect that the grid is updated with a command and this ran in the same frame
                    // The fix may be to mutate the grid resource directly
                    // As is though, this may actually work
                    continue;
                }
                let (_, mut pos) = maybe_pill.unwrap();

                // Exit early if the pill is already at the bottom
                if pos.row == 0 {
                    continue;
                }

                // The same cell after movement
                let (row, column) = (pos.row, pos.column);
                let next_cell = next_grid.get(row as usize, column as usize);
                
                // If the entity is not the same as the one in the next cell
                // this means the pill in that cell is moving down
                let mut move_down = false;
                match next_cell {
                    Cell::Pill(next_ent, _, _) => {
                        if next_ent != *ent {
                            move_down = true;
                        }
                    }
                    _ => {
                        move_down = true;
                    }
                }

                if move_down {
                    pills_moved = true;
                    pos.row -= 1;
                }

            }
            _ => {}
        }
    }
    if !pills_moved {
        if let Ok(ent) = control_query.get_single() {
            commands.entity(ent).remove::<Controllable>();
        }
        state.set(GameState::Resolving);
    }
    commands.insert_resource(next_grid);

}

fn reset_fall_time(
    mut time: ResMut<FixedTime>,
) {
    time.period = bevy::utils::Duration::from_secs(1);
}

fn adjust_fall_time(
    mut time: ResMut<FixedTime>,
    input: Res<Input<KeyCode>>,
) {
    if input.pressed(KeyCode::Down) {
        time.period = bevy::utils::Duration::from_secs_f32(0.2);
    } else if input.just_released(KeyCode::Down) {
        time.period = bevy::utils::Duration::from_secs_f32(1.);
    }
}

/// Put components here
/// 
#[derive(Component)]
struct GridPosition {
    row: u8,
    column: u8,
}

#[derive(Component)]
struct Virus(GridColor);

#[derive(Component)]
struct Pill(GridColor);

#[derive(Component)]
struct Controllable;

#[derive(Component)]
struct Board;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
enum GameState {
    #[default]
    NotStarted,
    PillDropping,
    Resolving,
    PillsFalling,
    Finished,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state::<GameState>()
        .add_systems(Startup, (setup_resources, setup_camera, spawn_board))
        .add_systems(PostStartup, start_game)
        .add_systems(OnExit(GameState::NotStarted), spawn_viruses)
        .add_systems(OnEnter(GameState::PillDropping), spawn_pill)
        .add_systems(OnExit(GameState::PillDropping), reset_fall_time)
        .add_systems(OnEnter(GameState::Resolving), clear_matches)
        .add_systems(
            Update, 
            (
                add_sprites, 
                (move_pill, rotate_pill, adjust_fall_time).run_if(in_state(GameState::PillDropping)),
                bevy::window::close_on_esc))
        .add_systems(
            FixedUpdate, 
            (
                move_pills
                    .run_if(in_state(GameState::PillDropping)
                    .or_else(in_state(GameState::PillsFalling))),
                ))
        .insert_resource(FixedTime::new_from_secs(1.))
        .add_systems(PostUpdate, update_transforms.before(bevy::transform::TransformSystem::TransformPropagate))
        .run();
}
