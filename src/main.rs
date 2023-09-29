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

fn spawn_board_background(
    mut commands: Commands,
    config: Res<GameConfig>,
) {
    let (rows, cols) = config.board_size;
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(CELL_SIZE * cols as f32, CELL_SIZE * rows as f32)),
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
    config: Res<GameConfig>,
    query: Query<Entity, With<GameBoard>>,
) {
    commands.entity(query.single()).with_children(|parent| {
        let mut virus_count = config.max_viruses;
        for row in 0..(board.rows-1) as u8 {
            // 3 is 100% for a virus cell, 300 is 1%
            //let upper = u32::pow(3, row as u32 + 1);
            let upper = 3 * (row as u32 + 1);
            // This starts the generation from the middle and works outwards
            let first_half = 0..(board.cols as u8)/2;
            let second_half = (board.cols as u8)/2+1..(board.cols as u8);
            let columns: Vec<u8> = first_half.rev()
                .zip(second_half)
                .collect::<Vec<_>>().iter()
                .flat_map(|tup| std::iter::once(tup.0).chain(std::iter::once(tup.1))).collect();
            for column in columns {
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
                        virus_count -= 1;
                    },
                    _ => {}
                }
                if virus_count == 0 {
                    return;
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

fn add_pill_to_board(
    mut commands: Commands,
    mut board: ResMut<Board<Entity>>,
    pieces_query: Query<(Entity, &Pill, &NextPill), Without<BoardPosition>>,
    board_query: Query<Entity, With<GameBoard>>,
) {
    let (row, col) = (board.rows-1, board.cols/2-1);
    let board_ent = board_query.single();
    for (ent, pill, pill_index) in pieces_query.iter() {
        let col = col + pill_index.0 as usize;
        let orientation = if pill_index.0 == 0 { Some(Orientation::Right) } else { Some(Orientation::Left) };
        board.set(row, col, Cell::Pill(ent, pill.0, orientation));
        commands.entity(ent)
            .remove::<NextPill>()
            .insert(BoardPosition { row: row as u8, column: col as u8 })
            .set_parent(board_ent);
        if pill_index.0 == 1 {
            commands.entity(ent).insert(Controllable);
        }
    }
}

fn spawn_pill(
    mut commands: Commands,
) {
    commands.spawn_batch([
        (Pill(rand_color()), NextPill(0)),
        (Pill(rand_color()), NextPill(1)),
    ]);
}

fn _spawn_pill(
    mut commands: Commands,
    mut board: ResMut<Board<Entity>>,
    query: Query<Entity, With<GameBoard>>,
){
    commands.entity(query.single()).with_children(|parent| {
        let (row, col) = (board.rows-1, board.cols/2-1);
        board.set(
            row,
            col,
            {
                let color = rand_color();
                let ent = parent.spawn((
                    Pill(color),
                    BoardPosition {row: row as u8, column: col as u8},
                    Controllable,
                )).id();
                Cell::Pill(ent, color, Some(Orientation::Right))
            }
        );
        board.set(
            row, 
            col+1,
            {
                let color = rand_color();
                let ent = parent.spawn((
                    Pill(color),
                    BoardPosition { row: row as u8, column: col as u8 + 1 },
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
    mut pill_query: Query<(Entity, &Pill, Option<&BoardPosition>, Option<&NextPill>), Added<Pill>>,
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
    for (entity, pill_type, board_position, next_pill) in pill_query.iter_mut() {
        let (mesh, material) = match pill_type.0 {
            CellColor::RED => (mesh_handles.1.clone().into(), material_handles.0.clone()),
            CellColor::BLUE => (mesh_handles.1.clone().into(), material_handles.1.clone()),
            CellColor::YELLOW => (mesh_handles.1.clone().into(), material_handles.2.clone()),
            CellColor::GREEN => (mesh_handles.1.clone().into(), material_handles.0.clone()),
            CellColor::ORANGE => (mesh_handles.1.clone().into(), material_handles.0.clone()),
            CellColor::PURPLE => (mesh_handles.1.clone().into(), material_handles.0.clone()),
        };
        let (x, y) = match (board_position, next_pill) {
            (Some(pos), _) => (pos.column as f32 * CELL_SIZE, pos.row as f32 * CELL_SIZE),
            (None, Some(next_index)) => (CELL_SIZE*(10.0 + next_index.0 as f32), 0.0),
            _ => continue,
        };
        commands.entity(entity)
            .insert(MaterialMesh2dBundle { 
                mesh,
                material, 
                transform: Transform::from_translation(Vec3::new(x, y, 100.0)),
                ..default() 
        });
    }
}

fn update_transforms(
    mut query: Query<(&BoardPosition, &mut Transform), Changed<BoardPosition>>,
    board: Res<Board<Entity>>,
) {
    for (board_position, mut transform) in query.iter_mut() {
        transform.translation.x = (board_position.column as f32 * CELL_SIZE) - (CELL_SIZE * board.cols as f32) / 2.0 + CELL_SIZE / 2.0;
        transform.translation.y = (board_position.row as f32 * CELL_SIZE) - (CELL_SIZE * board.rows as f32) / 2.0 + CELL_SIZE / 2.0;
    }
}

fn start_game(
    mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
    config: Res<GameConfig>,
) {
    let (rows, cols) = config.board_size;
    commands.insert_resource(Board::<Entity>::new(rows as usize, cols as usize));
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

fn check_movement_input(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut move_timer: ResMut<MovementTimer>,
    mut move_events: EventWriter<MoveEvent>,
    mut drop_timer: ResMut<DropTimer>,
    mut drop_events: EventWriter<DropEvent>,
) {
    if input.just_pressed(KeyCode::Left) || input.just_pressed(KeyCode::Right) {
        move_timer.reset();
        if input.just_pressed(KeyCode::Left) {
            move_events.send(MoveEvent::Left);
        }
        if input.just_pressed(KeyCode::Right) {
            move_events.send(MoveEvent::Right);
        }
    } else if input.pressed(KeyCode::Left) || input.pressed(KeyCode::Right) {
        move_timer.tick(time.delta());
        if move_timer.just_finished() {
            // Fire the event
            if input.pressed(KeyCode::Left) {
                move_events.send(MoveEvent::Left);
            }
            if input.pressed(KeyCode::Right) {
                move_events.send(MoveEvent::Right);
            }
        }
    }

    drop_timer.tick(time.delta());
    if input.just_pressed(KeyCode::Down) {
        drop_timer.reset();
        drop_events.send(DropEvent);
    } else if input.pressed(KeyCode::Down) {
        if drop_timer.just_finished() {
            drop_events.send(DropEvent);
            drop_timer.reset();
        }
        drop_timer.tick(core::time::Duration::from_secs_f32(0.1));
    }
    if drop_timer.just_finished() {
        drop_events.send(DropEvent);
    }
}

fn move_pill(
    mut control_query: Query<&mut BoardPosition, (With<Pill>, With<Controllable>)>,
    mut query: Query<&mut BoardPosition, (With<Pill>, Without<Controllable>)>,
    mut board: ResMut<Board<Entity>>,
    mut events: EventReader<MoveEvent>,
) {
    if events.len() < 1 { return }
    let from = { 
        if let Ok(pos) = control_query.get_single() {
            (pos.row as usize, pos.column as usize) 
        } else { return }
    };
    let to = {
        let mut pos = from;
        for event in events.iter() {
            match event {
                MoveEvent::Left => {
                    if pos.1 > 0 {
                        pos.1 -= 1;
                    }
                },
                MoveEvent::Right => {
                    if pos.1 < board.cols - 1 {
                        pos.1 += 1;
                    }
                },
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
    mut drop_events: EventReader<DropEvent>,
) { 
    if drop_events.len() < 1 { return }
    drop_events.clear();
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
    board_query: Query<Entity, With<GameBoard>>,
    next_pill: Query<Entity, With<NextPill>>,
) {
    for ent in board_query.iter() {
        commands.entity(ent).despawn_descendants();
    }
    for ent in next_pill.iter() {
        commands.entity(ent).despawn_recursive();
    }
}

fn reset_game(
    mut state: ResMut<NextState<GameState>>,
    mut config: ResMut<GameConfig>,
    input: Res<Input<KeyCode>>,
) {
    if input.pressed(KeyCode::Return) {
        config.max_viruses *= 2;
        state.set(GameState::Starting);
    }
}

fn pause_game(
    mut state: ResMut<NextState<AppState>>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) {
        state.set(AppState::Menu);
    }
}

fn startup_finished(
    mut game_state: ResMut<NextState<GameState>>,
) {
    game_state.set(GameState::Starting);
}

fn setup_menu(
    mut commands: Commands
) {
    let button_entity = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()},
            background_color: Color::BLACK.into(),
            ..default()})
        .with_children(|parent| {
            parent.spawn(ButtonBundle {
                style: Style {
                    width: Val::Px(200.0),
                    height: Val::Px(80.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                border_color: Color::WHITE.into(),
                background_color: Color::BLACK.into(),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn( TextBundle::from_section(
                    "Play",
                    TextStyle {
                        font_size: 40.0,
                        color: Color::WHITE.into(),
                        ..default()
                    }));
                });
            })
            .id();
    commands.insert_resource(MenuData { button_entity });

}

fn menu(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
    mut app_state: ResMut<NextState<AppState>>,
){
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = Color::GREEN.into();
                app_state.set(AppState::InGame);
            },
            Interaction::Hovered => {
                *color = Color::YELLOW.into();
            },
            Interaction::None => {
                *color = Color::BLACK.into();
            },
        }
    }
}

fn cleanup_menu(
    mut commands: Commands,
    menu: Res<MenuData>,
) {
    commands.entity(menu.button_entity).despawn_recursive();
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
struct NextPill(u8);

#[derive(Component)]
struct Controllable;

#[derive(Component)]
struct GameBoard;

#[derive(Resource, Deref, DerefMut)]
struct MovementTimer(Timer);

#[derive(Resource, Deref, DerefMut)]
struct DropTimer(Timer);

#[derive(Resource)]
struct MenuData {
    button_entity: Entity,
}

#[derive(Resource)]
struct GameConfig {
    board_size: (usize, usize),
    max_viruses: usize,
    drop_period: f32,
    fall_period: f32,
}

#[derive(Debug, Event)]
enum MoveEvent {
    Left,
    Right,
}

#[derive(Debug, Event)]
struct DropEvent;

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

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
enum AppState {
    #[default]
    Menu,
    InGame,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state::<AppState>()
        .add_state::<GameState>()
        .add_event::<MoveEvent>()
        .add_event::<DropEvent>()
        .insert_resource(GameConfig {
            board_size: (16, 8),
            max_viruses: 1,
            drop_period: 0.6,
            fall_period: 0.2,
        })
        .insert_resource(MovementTimer(Timer::from_seconds(0.2, TimerMode::Repeating)))
        .insert_resource(DropTimer(Timer::from_seconds(0.8, TimerMode::Repeating)))
        .insert_resource(FixedTime::new_from_secs(0.2))
        .add_systems(Startup, (setup_resources, setup_camera, spawn_board_background))
        .add_systems(PostStartup, startup_finished)
        .add_systems(OnEnter(AppState::Menu), setup_menu)
        .add_systems(OnExit(AppState::Menu), cleanup_menu)
        .add_systems(OnEnter(GameState::Starting), start_game)
        .add_systems(OnExit(GameState::Starting), (spawn_viruses, spawn_pill))
        .add_systems(OnEnter(GameState::PillDropping), (add_pill_to_board.before(spawn_pill), spawn_pill))
        .add_systems(OnEnter(GameState::Resolving), clear_matches)
        .add_systems(OnExit(GameState::Resolving), check_for_game_over)
        .add_systems(OnExit(GameState::Finished), cleanup_game)
        .add_systems(
            Update, 
            (
                add_sprites, 
                (move_pill, rotate_pill, drop_pill, check_movement_input)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(GameState::PillDropping)),
                pause_game
                    .run_if(in_state(AppState::InGame)),
                reset_game
                    .run_if(in_state(GameState::Finished)),
                menu
                    .run_if(in_state(AppState::Menu)),
                bevy::window::close_on_esc))
        .add_systems(
            FixedUpdate, 
            drop_pieces
                    .run_if(in_state(GameState::PiecesFalling))
                    .run_if(in_state(AppState::InGame)))
        .add_systems(PostUpdate, update_transforms.before(bevy::transform::TransformSystem::TransformPropagate))
        .run();
}