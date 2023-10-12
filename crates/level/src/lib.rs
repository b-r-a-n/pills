use bevy::prelude::*;
use pills_core::*;
use pills_input::*;
use rand::Rng;
use rand::rngs::ThreadRng;

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
            .add_systems(Update, check_for_completion
                .run_if(in_state(GameState::Active)))
            .add_systems(OnEnter(GameState::Finished), despawn_level)
        ;
    }
}

enum Outcome {
    None,
    Win(Entity),
    Loss(Entity),
    Draw,
}

#[derive(Clone)]
pub enum LevelDifficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Resource)]
pub struct Level {
    pub root: Option<Entity>,
    pub board_configs: Vec<Entity>,
    terminal_condition: TerminalCondition,
    outcome: Outcome,
}

#[derive(Clone, Component)]
pub struct LevelConfig {
    pub difficulty: LevelDifficulty,
}

fn random_config(difficulty: LevelDifficulty, rng: &mut ThreadRng) -> BoardConfig {
    let mut config = BoardConfig::default();
    match difficulty {
        LevelDifficulty::Easy => {
            config.drop_period = 0.8;
            config.max_viruses = rng.gen_range(4..=6);
        },
        LevelDifficulty::Medium => {
            config.drop_period = 0.6;
            config.max_viruses = rng.gen_range(12..=18);
        }
        LevelDifficulty::Hard => {
            config.drop_period = 0.4;
            config.max_viruses = rng.gen_range(36..=54);
        }
    }
    config
}

pub fn spawn_random_single_board_level(commands: &mut Commands, difficulty: LevelDifficulty, rng: &mut ThreadRng) -> Entity {
    let board_entity = commands
        .spawn((
            random_config(difficulty, rng),
            KeyControlled,
        ))
        .id();
    commands
        .insert_resource(Level {
            root: None,
            board_configs: vec![board_entity],
            terminal_condition: TerminalCondition::NoneRemaining,
            outcome: Outcome::None,
        });
    board_entity
}

pub fn spawn_single_board_level(commands: &mut Commands) -> Entity {
    let board_entity = commands
        .spawn((BoardConfig::default(), KeyControlled))
        .id();
    commands
        .insert_resource(Level {
            root: None,
            board_configs: vec![board_entity],
            terminal_condition: TerminalCondition::NoneRemaining,
            outcome: Outcome::None,
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
    if let Some(root) = level.root {
        info!("\t Despawning root {:?}", root);
        commands.entity(root).despawn_recursive();
    }
    commands.remove_resource::<Level>();
}

fn check_for_completion(
    mut state: ResMut<NextState<GameState>>,
    mut level: ResMut<Level>,
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
            state.set(GameState::Finished);
            if n == 1 { level.outcome = Outcome::Win(e); } else { level.outcome = Outcome::Draw; } 
        },
        (_, Some(e), n, _) => { 
            state.set(GameState::Finished);
            if n == 1 { level.outcome = Outcome::Loss(e); } else { level.outcome = Outcome::Draw; } 
        },
        (_, _, _, 0) => { 
            state.set(GameState::Finished);
            level.outcome = Outcome::Draw
        },
        _ => {},
    }
}