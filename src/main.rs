use bevy::prelude::*;
use rand::{Rng, thread_rng};

use pills_game_board::*;

#[derive(Component, Deref, DerefMut)]
struct CellComponent(Cell<Entity>);

#[derive(Resource, Deref, DerefMut)]
struct GameBoard(Board<Entity>);

/// Put systems here
/// 
fn setup_camera(
    mut commands: Commands
) {
    commands.spawn(Camera2dBundle::default());
}

const RED_COLOR : Color = Color::rgb(255.0/255.0, 115.0/255.0, 106.0/255.0);
const YELLOW_COLOR : Color = Color::rgb(255.0/255.0, 213.0/255.0, 96.0/255.0);
const BLUE_COLOR : Color = Color::rgb(0.0/255.0, 194.0/255.0, 215.0/255.0);

#[derive(Resource, Deref, DerefMut)]
struct PieceAtlasHandle(Handle<TextureAtlas>);

fn setup_resources(
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
) {
    let texture_handle = asset_server.load("textures/pieces.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 1, 6, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.insert_resource(PieceAtlasHandle(texture_atlas_handle));

    let move_sound_handle = asset_server.load("sounds/move.ogg");
    let rotate_sound_handle = asset_server.load("sounds/rotate.ogg");
    let pop_sound_handle = asset_server.load("sounds/pop.ogg");
    commands.insert_resource(PillSounds { move_sound_handle, rotate_sound_handle, pop_sound_handle });
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
        ActiveGameBoard,
    ));

}

fn spawn_viruses(
    mut commands: Commands,
    mut board: ResMut<GameBoard>,
    config: Res<GameConfig>,
    query: Query<Entity, With<ActiveGameBoard>>,
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
    mut board: ResMut<GameBoard>,
    mut result_events: EventWriter<GameResult>,
    mut spawn_events: EventWriter<PillSpawned>,
    pieces_query: Query<(Entity, &Pill, &NextPill), Without<BoardPosition>>,
    board_query: Query<Entity, With<ActiveGameBoard>>,
) {
    let (row, col) = (board.rows-1, board.cols/2-1);
    let board_ent = board_query.single();
    for (ent, pill, pill_index) in pieces_query.iter() {
        let col = col + pill_index.0 as usize;
        let orientation = if pill_index.0 == 0 { Some(Orientation::Right) } else { Some(Orientation::Left) };
        if board.get(row, col) != Cell::Empty {
            result_events.send(GameResult::Loss);
            return;
        }
        board.set(row, col, Cell::Pill(ent, pill.0, orientation));
        commands.entity(ent)
            .remove::<NextPill>()
            .insert(BoardPosition { row: row as u8, column: col as u8 })
            .set_parent(board_ent);
        if pill_index.0 == 1 {
            commands.entity(ent).insert(Controllable);
            spawn_events.send(PillSpawned);
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

fn add_sprites(
    mut commands: Commands,
    atlas_handle: Res<PieceAtlasHandle>,
    mut virus_query: Query<(Entity, &Virus), (Added<Virus>, With<BoardPosition>)>,
    mut pill_query: Query<(Entity, &Pill, Option<&BoardPosition>, Option<&NextPill>), Added<Pill>>,
    cleared_query: Query<(Entity, &ClearedCell), (Added<ClearedCell>, With<BoardPosition>)>,
) {
    for (entity, virus_type) in virus_query.iter_mut() {
        let (texture_atlas, sprite) = match virus_type.0 {
            CellColor::RED => (atlas_handle.clone(), TextureAtlasSprite::new(1)),
            CellColor::BLUE => (atlas_handle.clone(), TextureAtlasSprite::new(0)),
            CellColor::YELLOW => (atlas_handle.clone(), TextureAtlasSprite::new(2)),
            CellColor::GREEN => (atlas_handle.clone(), TextureAtlasSprite::new(2)),
            CellColor::ORANGE => (atlas_handle.clone(), TextureAtlasSprite::new(2)),
            CellColor::PURPLE => (atlas_handle.clone(), TextureAtlasSprite::new(1)),
        };
        let transform = Transform::from_scale(Vec3::new(0.5, 0.5, 100.0));
        commands.entity(entity)
            .insert(SpriteSheetBundle { 
                texture_atlas,
                sprite, 
                transform,
                ..default() 
        });
    }
    for (entity, pill_type, board_position, next_pill) in pill_query.iter_mut() {
        let color = match pill_type.0 {
            CellColor::RED => RED_COLOR,
            CellColor::YELLOW => YELLOW_COLOR,
            CellColor::BLUE => BLUE_COLOR,
            CellColor::ORANGE => RED_COLOR,
            CellColor::GREEN => YELLOW_COLOR,
            CellColor::PURPLE => BLUE_COLOR,
        };
        let sprite = TextureAtlasSprite {index:5, color, ..default()};
        let transform = match (board_position, next_pill) {
            (Some(pos), _) => { 
                Transform::from_xyz(pos.column as f32 * CELL_SIZE, pos.row as f32 * CELL_SIZE, 100.0)
                    .with_scale(Vec3::new(0.5, 0.5, 1.0))
            },
            (None, Some(next_index)) => {
                Transform::from_xyz((10.0 + next_index.0 as f32) * CELL_SIZE, 0.0, 100.0)
                    .with_scale(Vec3::new(0.5, 0.5, 1.0))
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2 * ((next_index.0 as f32 * 2.0) + 1.)))
            },
            _ => continue,
        };
        commands.entity(entity)
            .insert(SpriteSheetBundle { 
                texture_atlas: atlas_handle.clone(),
                sprite,
                transform,
                ..default() 
        });
    }
    for (entity, cell) in &cleared_query {
        let color = match cell.0 {
            CellColor::RED => RED_COLOR,
            CellColor::YELLOW => YELLOW_COLOR,
            CellColor::BLUE => BLUE_COLOR,
            CellColor::ORANGE => RED_COLOR,
            CellColor::GREEN => YELLOW_COLOR,
            CellColor::PURPLE => BLUE_COLOR,
        };
        commands.entity(entity)
            .insert(SpriteSheetBundle {
                sprite: TextureAtlasSprite { index: 3, color, ..default()},
                texture_atlas: atlas_handle.clone(),
                transform: Transform::from_scale(Vec3::new(0.5, 0.5, 1.0)),
                ..default()
            });
    }
}

fn update_transforms(
    mut query: Query<(&BoardPosition, &mut Transform), Or<(Added<BoardPosition>, Changed<BoardPosition>)>>,
    board: Res<GameBoard>,
) {
    for (board_position, mut transform) in query.iter_mut() {
        transform.translation.x = (board_position.column as f32 * CELL_SIZE) - (CELL_SIZE * board.cols as f32) / 2.0 + CELL_SIZE / 2.0;
        transform.translation.y = (board_position.row as f32 * CELL_SIZE) - (CELL_SIZE * board.rows as f32) / 2.0 + CELL_SIZE / 2.0;
        if let Some(orientation) = board
            .get(board_position.row as usize, board_position.column as usize)
            .get_orientation() {
            transform.rotation = match orientation {
                Orientation::Above => Quat::from_rotation_z(std::f32::consts::PI),
                Orientation::Below => Quat::from_rotation_z(0.),
                Orientation::Left => Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
                Orientation::Right => Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
            };
        }
    }
}

fn start_game(
    mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
    config: Res<GameConfig>,
) {
    let (rows, cols) = config.board_size;
    commands.insert_resource(MovementTimer(Timer::from_seconds(0.2, TimerMode::Repeating)));
    commands.insert_resource(DropTimer(Timer::from_seconds(config.drop_period, TimerMode::Repeating)));
    commands.insert_resource(FixedTime::new_from_secs(config.fall_period));
    commands.insert_resource(GameBoard(Board::new(rows as usize, cols as usize)));
    state.set(GameState::PillDropping);
}

fn check_resolve_timer(
    mut state: ResMut<NextState<GameState>>,
    mut timer: ResMut<ResolveTimer>,
    time: Res<Time>,
) {
    if timer.tick(time.delta()).just_finished() {
        state.set(GameState::PiecesFalling);
    }
}

fn clear_matches(
    mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
    mut board: ResMut<GameBoard>,
    mut clear_events: EventWriter<ClearEvent>,
    mut virus_clear_events: EventWriter<VirusCleared>,
) {
    let mut cells_cleared = false;
    let next_board = board.resolve(|lhs, rhs| lhs.color() == rhs.color());
    for i in 0..board.cells.len() {
        if next_board.cells[i] == Cell::Empty {
            match board.cells[i] {
                Cell::Virus(ent, color) => {
                    virus_clear_events.send(VirusCleared(color));
                    commands.entity(ent).despawn_recursive();
                    commands.spawn((
                        BoardPosition { row: (i / board.cols) as u8, column: (i % board.cols) as u8 },
                        ClearedCell(color),
                    ));
                    cells_cleared = true;
                }
                Cell::Pill(ent, color, _) => {
                    commands.entity(ent).despawn_recursive();
                    commands.spawn((
                        BoardPosition { row: (i / board.cols) as u8, column: (i % board.cols) as u8 },
                        ClearedCell(color),
                    ));
                    cells_cleared = true;
                }
                _ => {}
            }
        }
        board.cells[i] = next_board.cells[i];
    }
    if cells_cleared {
        clear_events.send(ClearEvent);
    } else {
        state.set(GameState::PillDropping);
    }
    commands.insert_resource(ResolveTimer(Timer::from_seconds(0.2, TimerMode::Once)));
}

fn rotate_pill(
    mut control_query: Query<&mut BoardPosition, (With<Pill>, With<Controllable>)>,
    mut query: Query<&mut BoardPosition, (With<Pill>, Without<Controllable>)>,
    mut board: ResMut<GameBoard>,
    mut pill_moved_events: EventWriter<PillEvent>,
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
        pill_moved_events.send(PillEvent::PillRotated);
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
    mut board: ResMut<GameBoard>,
    mut events: EventReader<MoveEvent>,
    mut pill_moved_events: EventWriter<PillEvent>,
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

        pill_moved_events.send(PillEvent::PillMoved);
    }
}

fn drop_pieces(
    mut board: ResMut<GameBoard>,
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
        *board = GameBoard(next_board);
    } else {
        state.set(GameState::Resolving);
    }
}

fn drop_pill(
    mut commands: Commands,
    mut board: ResMut<GameBoard>,
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
    mut event: EventWriter<GameResult>,
    board: Res<GameBoard>,
) {
    if board.virus_count() < 1 {
        event.send(GameResult::Win);
    }
}

fn handle_game_result(
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    mut event: EventReader<GameResult>,
) {
    if event.is_empty() { return }
    match event.iter().next().unwrap() {
        GameResult::Loss => {
            commands.spawn(MenuTitle::GameOver);
            commands.spawn_batch([
                (MenuOption::Play),
                (MenuOption::Exit)
            ]);
            
        },
        GameResult::Win => {
            commands.spawn(MenuTitle::Victory);
            commands.spawn_batch([
                (MenuOption::NextLevel),
                (MenuOption::Exit)
            ]);
        },
    }
    game_state.set(GameState::Finished);
}

fn handle_finished(
    mut app_state: ResMut<NextState<AppState>>,
) {
    app_state.set(AppState::Menu);
}

fn cleanup_cleared(
    mut commands: Commands,
    query: Query<Entity, With<ClearedCell>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn cleanup_game(
    mut commands: Commands,
    board_query: Query<Entity, With<ActiveGameBoard>>,
    next_pill: Query<Entity, With<NextPill>>,
) {
    for ent in board_query.iter() {
        commands.entity(ent).despawn_descendants();
    }
    for ent in next_pill.iter() {
        commands.entity(ent).despawn_recursive();
    }
}

fn pause_game(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) {
        commands.spawn(MenuOption::Play);
        state.set(AppState::Menu);
    }
}

fn startup_finished(
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
) {
    commands.spawn_batch([
        (MenuOption::Play),
        (MenuOption::Exit),
    ]);
    game_state.set(GameState::Starting);
}

#[derive(Component)]
enum MenuOption {
    Play,
    NextLevel,
    Exit,
}

#[derive(Component)]
enum MenuTitle {
    GameOver,
    Victory,
}

impl Into<String> for &MenuOption {
    fn into(self) -> String {
        match self {
            MenuOption::Play => "Play".to_string(),
            MenuOption::NextLevel => "Next Level".to_string(),
            MenuOption::Exit => "Exit".to_string(),
        }
    }
}

fn setup_ui(
    mut commands: Commands,
) {
    let ui_entity = commands.spawn(NodeBundle {
        style: Style {
            height: Val::Px(100.0),
            width: Val::Px(200.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        background_color: Color::BLACK.into(),
        ..default()
    }).id();
    commands.spawn((
        TextBundle::from_section(
            format!("Score: {}", 0),
            TextStyle {
                font_size: 40.0,
                color: Color::WHITE.into(),
                ..default()
        }),
        Score(0),
    ))
        .set_parent(ui_entity);
}

fn setup_menu(
    mut commands: Commands,
    query: Query<(Entity, &MenuOption)>,
    title_query: Query<(Entity, &MenuTitle)>,
) {
    let button_entity = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()},
            background_color: Color::BLACK.into(),
            ..default()})
        .id();
    if let Ok((entity, title)) = title_query.get_single() {
        commands.entity(entity).insert(
            TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: match title { 
                            MenuTitle::GameOver => "Game Over".to_string(), 
                            MenuTitle::Victory => "Victory".to_string()
                        },
                        style: TextStyle {
                            font_size: 80.0,
                            color: Color::WHITE.into(),
                            ..default()
                        },
                    }],
                    alignment: TextAlignment::Center,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                },
                ..default()
            })
            .set_parent(button_entity);
    }
    for (entity, option) in &query {
        commands.entity(entity).insert(ButtonBundle {
            style: Style {
                width: Val::Px(400.0),
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
                option,
                TextStyle {
                    font_size: 40.0,
                    color: Color::WHITE.into(),
                    ..default()
                }));
            })
        .set_parent(button_entity);
    }
    commands.insert_resource(MenuData { button_entity });

}

fn menu(
    mut commands: Commands,
    mut interaction_query: Query<(&Interaction, &MenuOption, &mut BackgroundColor, &Children), (Changed<Interaction>, With<Button>)>,
    mut text_query: Query<&mut Text>,
    mut app_state: ResMut<NextState<AppState>>,
    curr_game_state: Res<State<GameState>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut config: ResMut<GameConfig>,
    focused_windows: Query<(Entity, &Window)>,
){
    for (interaction, option, mut background_color, children) in &mut interaction_query {
        match (interaction, option) {
            (Interaction::Pressed, MenuOption::Play) => {
                let mut text = text_query.get_mut(children[0]).unwrap();
                *background_color = Color::DARK_GRAY.into();
                text.sections[0].style.color = Color::PINK.into();
                if curr_game_state.get() == &GameState::Finished {
                    game_state.set(GameState::Starting);
                }
                app_state.set(AppState::InGame);
            },
            (Interaction::Pressed, MenuOption::NextLevel) => {
                let mut text = text_query.get_mut(children[0]).unwrap();
                *background_color = Color::DARK_GRAY.into();
                text.sections[0].style.color = Color::PINK.into();
                game_state.set(GameState::Starting);
                app_state.set(AppState::InGame);
            },
            (Interaction::Pressed, MenuOption::Exit) => {
                let mut text = text_query.get_mut(children[0]).unwrap();
                *background_color = Color::DARK_GRAY.into();
                text.sections[0].style.color = Color::PINK.into();
                for (window, focus) in focused_windows.iter() {
                    if !focus.focused {
                        continue;
                    }
                    commands.entity(window).despawn();
                }
            },
            (Interaction::Hovered, _) => {
                let mut text = text_query.get_mut(children[0]).unwrap();
                *background_color = Color::BLACK.into();
                text.sections[0].style.color = Color::YELLOW.into();
            },
            (Interaction::None, _) => {
                let mut text = text_query.get_mut(children[0]).unwrap();
                *background_color = Color::BLACK.into();
                text.sections[0].style.color = Color::WHITE.into();
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

#[derive(Event)]
enum PillEvent {
    PillMoved,
    PillRotated,
}

#[derive(Resource)]
struct PillSounds{
    move_sound_handle: Handle<AudioSource>, 
    rotate_sound_handle: Handle<AudioSource>,
    pop_sound_handle: Handle<AudioSource>,
}

fn play_pill_sound(
    mut commands: Commands,
    mut events: EventReader<PillEvent>,
    sound_handles: Res<PillSounds>,
) {
    if events.is_empty() { return }
    for event in events.iter() {
        match event {
            PillEvent::PillMoved => {
                commands.spawn(AudioBundle {
                    source: sound_handles.move_sound_handle.clone(),
                    settings: PlaybackSettings::DESPAWN,
                });
            }
            PillEvent::PillRotated =>  {
                commands.spawn(AudioBundle {
                    source: sound_handles.rotate_sound_handle.clone(),
                    settings: PlaybackSettings::DESPAWN,
                });
            }
        }
    }
    events.clear();
}

fn play_clear_sound(
    mut commands: Commands,
    mut events: EventReader<ClearEvent>,
    sound_handles: Res<PillSounds>,
) {
    if events.is_empty() { return }
    commands.spawn(AudioBundle {
        source: sound_handles.pop_sound_handle.clone(),
        settings: PlaybackSettings::DESPAWN,
    });
    events.clear();
}

fn update_score(
    mut query: Query<&mut Score>,
    mut spawn_events: EventReader<PillSpawned>,
    mut virus_cleared_events: EventReader<VirusCleared>,
) {
    if let Ok(mut score) = query.get_single_mut() {
        for _ in spawn_events.iter() {
            if score.0 > 0 {
                score.0 -= 1;
            }
        }
        for _ in virus_cleared_events.iter() {
            score.0 += 3;
        }
    }
}

fn update_ui(
    mut query: Query<(&mut Text, &Score), Changed<Score>>,
) {
    for (mut text, score) in &mut query {
        text.sections[0].value = format!("Score: {}", score.0);
    }
}

fn cleanup_ui(
    mut commands: Commands,
    query: Query<&Parent, With<Score>>,
) {
    let entity = query.single().get();
    commands.entity(entity).despawn_recursive();
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

#[derive(Component, Deref, DerefMut)]
struct ClearedCell(CellColor);

#[derive(Component)]
struct NextPill(u8);

#[derive(Component)]
struct Controllable;

#[derive(Component)]
struct ActiveGameBoard;

#[derive(Component, Deref, DerefMut)]
struct Score(usize);

#[derive(Resource, Deref, DerefMut)]
struct MovementTimer(Timer);

#[derive(Resource, Deref, DerefMut)]
struct DropTimer(Timer);

#[derive(Resource, Deref, DerefMut)]
struct ResolveTimer(Timer);

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

#[derive(Debug, Event)]
struct ClearEvent;

#[derive(Debug, Event)]
struct PillSpawned;

#[derive(Debug, Deref, DerefMut, Event)]
struct VirusCleared(CellColor);

#[derive(Event)]
enum GameResult { Loss, Win }

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
        .add_event::<PillEvent>()
        .add_event::<ClearEvent>()
        .add_event::<GameResult>()
        .add_event::<PillSpawned>()
        .add_event::<VirusCleared>()
        .insert_resource(GameConfig {
            board_size: (16, 8),
            max_viruses: 1,
            drop_period: 0.6,
            fall_period: 0.2,
        })
        .add_systems(Startup, (setup_resources, setup_camera, spawn_board_background.after(setup_resources)))
        .add_systems(PostStartup, startup_finished)
        .add_systems(OnEnter(AppState::Menu), setup_menu)
        .add_systems(OnExit(AppState::Menu), cleanup_menu)
        .add_systems(OnEnter(GameState::Starting), (start_game, setup_ui))
        .add_systems(OnExit(GameState::Starting), (spawn_viruses, spawn_pill))
        .add_systems(OnEnter(GameState::PillDropping), (add_pill_to_board.before(spawn_pill), spawn_pill))
        .add_systems(OnEnter(GameState::Resolving), clear_matches)
        .add_systems(OnExit(GameState::Resolving), (cleanup_cleared, check_for_game_over))
        .add_systems(OnEnter(GameState::Finished), handle_finished)
        .add_systems(OnExit(GameState::Finished), (cleanup_game, cleanup_ui))
        .add_systems(
            Update, 
            (
                add_sprites, 
                handle_game_result,
                update_score,
                update_ui,
                (move_pill, rotate_pill, drop_pill, check_movement_input, play_pill_sound)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(GameState::PillDropping)),
                (play_clear_sound, check_resolve_timer)
                    .run_if(in_state(GameState::Resolving)),
                pause_game
                    .run_if(in_state(AppState::InGame)),
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