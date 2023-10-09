use bevy::prelude::*;
use pills_core::*;
use core::time::Duration;

pub struct InputPlugin;

#[derive(Component)]
pub struct KeyControlled;

#[derive(Component, Deref, DerefMut)]
struct MovementTimer(Timer);

#[derive(Component, Deref, DerefMut)]
struct DropTimer(Timer);

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Starting), setup_key_control)
            .add_systems(Update, (
                handle_drop_input, 
                handle_movement_input, 
                handle_rotate_input)
                    .run_if(in_state(GameState::Active))
            )
        ;
    }
}

fn setup_key_control(
    mut commands: Commands,
    key_control_query: Query<(Entity, &BoardConfig)>,
) {
    for (board, config) in key_control_query.iter() {
        info!("Found a board with key control: {:?}", board);
        commands.entity(board)
            .insert(MovementTimer(Timer::from_seconds(0.2, TimerMode::Repeating)))
            .insert(DropTimer(Timer::from_seconds(config.drop_period, TimerMode::Repeating)));
    }
}

fn handle_movement_input(
    mut commands: Commands,
    mut movement_timer_query: Query<&mut MovementTimer, With<KeyControlled>>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    pivot_piece_query: Query<(Entity, &InBoard), With<PivotPiece>>,
) {
    for (piece_ent, board) in pivot_piece_query.iter() {
        if let Ok(mut timer) = movement_timer_query.get_mut(**board) {
            let (l, r) = (input.just_pressed(KeyCode::Left), input.just_pressed(KeyCode::Right));
            if (l || r) && !(l && r) {
                timer.reset();
                if l {
                    commands.entity(piece_ent).insert(Move::Left);
                } 
                if r {
                    commands.entity(piece_ent).insert(Move::Right);
                }
            }
            let (l, r) = (input.pressed(KeyCode::Left), input.pressed(KeyCode::Right));
            if (l || r) && !(l && r) {
                timer.tick(time.delta());
                if timer.just_finished() {
                    if l {
                        commands.entity(piece_ent).insert(Move::Left);
                    }
                    if r {
                        commands.entity(piece_ent).insert(Move::Right);
                    }
                }
            }
        }
    }
}

fn handle_drop_input(
    mut commands: Commands,
    mut drop_timer_query: Query<&mut DropTimer, With<KeyControlled>>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    pivot_piece_query: Query<(Entity, &InBoard), With<PivotPiece>>,
) {
    for (piece_ent, board) in pivot_piece_query.iter() {
        if let Ok(mut timer) = drop_timer_query.get_mut(**board) {
            timer.tick(time.delta());
            if input.just_pressed(KeyCode::Down) {
                timer.reset();
                commands.entity(piece_ent).insert(Drop);
            } else if input.pressed(KeyCode::Down) {
                if timer.just_finished() {
                    commands.entity(piece_ent).insert(Drop);
                    timer.reset();
                }
                timer.tick(Duration::from_secs_f32(0.1));
            }
            if timer.just_finished() {
                commands.entity(piece_ent).insert(Drop);
            }
        }
    }
}

fn handle_rotate_input(
    mut commands: Commands,
    key_control_query: Query<Entity, With<KeyControlled>>,
    input: Res<Input<KeyCode>>,
    pivot_piece_query: Query<(Entity, &InBoard), With<PivotPiece>>,
) {
    for (piece_ent, board) in pivot_piece_query.iter() {
        if key_control_query.get(**board).is_ok() {
            if input.just_pressed(KeyCode::Z) {
                commands.entity(piece_ent).insert(Rotate::Left);
            } else if input.just_pressed(KeyCode::X) {
                commands.entity(piece_ent).insert(Rotate::Right);
            }
        }
    }
}