use bevy::prelude::*;
use pills_core::*;
use pills_input::*;

pub enum TerminalCondition {
    FirstWin,
    FirstLoss,
    LastRemaining,
    NoneRemaining,
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<LevelFinished>()
            .add_systems(Update, check_for_completion
                .run_if(in_state(GameState::Active)))
            .add_systems(OnEnter(GameState::Finished), despawn_level
                //.run_if(resource_exists::<Level>())
        )
        ;
    }
}

#[derive(Event)]
pub enum LevelFinished {
    Win(Entity),
    Loss(Entity),
    Draw
}

#[derive(Resource)]
pub struct Level {
    pub board_configs: Vec<Entity>,
    terminal_condition: TerminalCondition,
}

pub fn spawn_single_board_level(commands: &mut Commands) -> Entity {
    let board_entity = commands
        .spawn((BoardConfig::default(), KeyControlled))
        .id();
    commands
        .insert_resource(Level {
            board_configs: vec![board_entity],
            terminal_condition: TerminalCondition::NoneRemaining,
        });
    board_entity
}

fn despawn_level(
    mut commands: Commands,
    query: Query<Entity, With<InBoard>>,
    level: Res<Level>,
) {
    info!("Despawning level");
    for entity in query.iter() {
        info!("\t Despawning InBoard entity {:?}", entity);
        commands.entity(entity).despawn_recursive();
    }
    for entity in level.board_configs.iter() {
        info!("\t Despawning board {:?}", entity);
        commands.entity(*entity).despawn_recursive();
    }
    commands.remove_resource::<Level>();
}

fn check_for_completion(
    mut events: EventWriter<LevelFinished>,
    level: Res<Level>,
    finished_boards: Query<&BoardFinished, With<GameBoard>>,
) {
    use TerminalCondition::*;
    use BoardFinished::*;
    let mut unfinished = 0;
    let mut finished = 0;
    let mut winner = None;
    let mut loser = None;
    for entity in level.board_configs.iter() {
        match (&level.terminal_condition, finished_boards.get(*entity)) {
            (FirstWin, Ok(Win)) => { finished += 1; winner = Some(*entity); },
            (FirstLoss, Ok(Loss)) => { finished += 1; loser = Some(*entity); },
            (LastRemaining, Err(_)) => { unfinished += 1; winner = Some(*entity); },
            (NoneRemaining, Ok(_)) => { finished += 1; },
            (_, Err(_)) => { unfinished += 1; },
            (_, Ok(_)) => { finished += 1; }
        }
    }
    match (winner, loser, finished, unfinished) {
        (Some(e), _, n, _) => { 
            if n == 1 { events.send(LevelFinished::Win(e)); } else { events.send(LevelFinished::Draw); } 
        },
        (_, Some(e), n, _) => { 
            if n == 1 { events.send(LevelFinished::Loss(e)); } else { events.send(LevelFinished::Draw); }
        },
        (_, _, _, 0) => { events.send(LevelFinished::Draw); },
        _ => {},
    }
}