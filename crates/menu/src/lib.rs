use bevy::prelude::*;
use bevy::ecs::system::EntityCommand;
use pills_core::*;
use pills_level::*;
use pills_score::*;

pub struct MenuPlugin;


impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state::<AppState>()
            .init_resource::<FinishedCount>()
            .add_systems(
                Update,
                (
                    (
                        pause_game, 
                    )
                        .run_if(in_state(AppState::InGame)),
                    menu
                        .run_if(in_state(AppState::Menu))
                )
            )
            .add_systems(Startup, startup_menu)
            .add_systems(OnEnter(AppState::Menu), setup_menu)
            .add_systems(OnExit(AppState::Menu), cleanup_menu)
            .add_systems(OnEnter(GameState::Finished), handle_level_finished)
        ;
    }
}

#[derive(Default, Deref, DerefMut, Resource)]
struct FinishedCount(usize);

#[derive(Clone, Component)]
enum MenuTitle {
    GameOver,
    Victory,
    Custom(String),
}

impl Into<String> for MenuTitle {
    fn into(self) -> String {
        match self {
            MenuTitle::GameOver => "Game Over".to_string(),
            MenuTitle::Victory => "Victory".to_string(),
            MenuTitle::Custom(text) => text.clone(),
        }
    }
}

#[derive(Clone, Component)]
enum MenuOption {
    Play,
    NextLevel,
    SpecificLevel,
    Exit,
}

impl Into<String> for MenuOption {
    fn into(self) -> String {
        match self {
            MenuOption::Play => "Play".to_string(),
            MenuOption::NextLevel => "Next Level".to_string(),
            MenuOption::SpecificLevel => "".to_string(),
            MenuOption::Exit => "Exit".to_string(),
        }
    }
}

impl EntityCommand for MenuOption {
    fn apply(self, id: Entity, world: &mut World) {
        let value = match self {
            Self::SpecificLevel => {
                match world.entity(id).get::<LevelConfig>().and_then(|config| Some(config.difficulty.clone())) {
                    Some(LevelDifficulty::Easy) => "Easy",
                    Some(LevelDifficulty::Medium) => "Medium",
                    Some(LevelDifficulty::Hard) => "Hard",
                    None => "",
                }.to_string()
            },
            _ => {
                self.into()
            },
        };
        world.entity_mut(id).insert(ButtonBundle {
            style: Style {
                width: Val::Px(400.0),
                height: Val::Px(80.0),
                border: UiRect::all(Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()})
        .with_children(|parent| {
            parent.spawn( TextBundle::from_section(
                value,
                TextStyle {
                    font_size: 40.0,
                    color: Color::WHITE.into(),
                    ..default()
                }));
            })
        ;
    }
}

impl EntityCommand for MenuTitle {
    fn apply(self, id: Entity, world: &mut World) {
        world.entity_mut(id).insert(TextBundle::from_section( 
            self,
            TextStyle {
                font_size: 80.0,
                color: Color::WHITE.into(),
                ..default()
            }))
        ;
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
enum AppState {
    #[default]
    Menu,
    InGame,
}

#[derive(Resource)]
struct MenuData {
    root_entity: Entity,
}

fn pause_game(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<GameState>>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) {
        commands.spawn(MenuTitle::Custom("Paused".to_string()));
        commands.spawn(MenuOption::Play);
        state.set(AppState::Menu);
        game_state.set(GameState::Paused);
    }
}

fn startup_menu(
    mut commands: Commands,
) {
    commands.spawn_batch([
        (MenuOption::Play),
        (MenuOption::Exit),
    ]);
}

fn setup_menu(
    mut commands: Commands,
    query: Query<(Entity, &MenuOption)>,
    title_query: Query<(Entity, &MenuTitle)>,
) {
    let root_entity = commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()},
            background_color: Color::BLACK.into(),
            ..default()})
        .id();
    for (entity, title) in &title_query {
        commands.entity(entity)
            .add(title.clone())
            .set_parent(root_entity);
    }
    for (entity, option) in &query {
        commands.entity(entity)
            .add(option.clone())
            .set_parent(root_entity);
    }
    commands.insert_resource(MenuData { root_entity });

}

fn menu(
    mut commands: Commands,
    mut interaction_query: Query<(Entity, &Interaction, &MenuOption, &mut BackgroundColor, &Children), (Changed<Interaction>, With<Button>)>,
    mut text_query: Query<&mut Text>,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<GameState>>,
    player_query: Query<Entity, With<Player>>,
    level_config_query: Query<&LevelConfig>,
    curr_game_state: Res<State<GameState>>,
    finished_count: Res<FinishedCount>,
    focused_windows: Query<(Entity, &Window)>,
){
    for (id, interaction, option, mut background_color, children) in &mut interaction_query {
        match (interaction, option) {
            (Interaction::Pressed, MenuOption::Play) => {
                match curr_game_state.get() {
                    GameState::Finished | GameState::NotStarted => {
                        let player_ent = player_query.single();
                        let board_ent = spawn_single_board_level(&mut commands);
                        commands.entity(board_ent).insert(BoardPlayer(player_ent));
                        game_state.set(GameState::Starting);
                    },
                    GameState::Paused => game_state.set(GameState::Active),
                    _ => {},
                }
                app_state.set(AppState::InGame);
            },
            (Interaction::Pressed, MenuOption::NextLevel) => {
                let mut difficulty = LevelDifficulty::Easy;
                if **finished_count > 2 {
                    difficulty = LevelDifficulty::Medium;
                }
                if **finished_count > 4 {
                    difficulty = LevelDifficulty::Hard;
                }
                let player_ent = player_query.single();
                let board_ent = spawn_random_single_board_level(&mut commands, difficulty, &mut rand::thread_rng());
                commands.entity(board_ent).insert(BoardPlayer(player_ent));
                game_state.set(GameState::Starting);
                app_state.set(AppState::InGame);
            },
            (Interaction::Pressed, MenuOption::SpecificLevel) => {
                if let Ok(level_config) = level_config_query.get(id){
                    let difficulty = level_config.difficulty.clone();
                    let player_ent = player_query.single();
                    let board_ent = spawn_random_single_board_level(&mut commands, difficulty, &mut rand::thread_rng());
                    commands.entity(board_ent).insert(BoardPlayer(player_ent));
                    game_state.set(GameState::Starting);
                    app_state.set(AppState::InGame);
                }

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
    commands.entity(menu.root_entity).despawn_recursive();
}

#[derive(Component)]
struct LastOption;

fn handle_level_finished(
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut finished_count: ResMut<FinishedCount>,
    boards: Query<(&BoardFinished, &BoardPlayer), With<BoardPlayer>>,
    scores: Query<&GlobalScore, With<Player>>,
) {
    let (result, player) = boards.single();
    match result {
        BoardFinished::Win => {
            **finished_count += 1;
            commands.spawn(MenuTitle::Victory);
            commands.spawn_batch([
                (MenuOption::SpecificLevel, LevelConfig { difficulty: LevelDifficulty::Easy }),
                (MenuOption::SpecificLevel, LevelConfig { difficulty: LevelDifficulty::Medium }),
                (MenuOption::SpecificLevel, LevelConfig { difficulty: LevelDifficulty::Hard }),
            ]);
            commands.spawn((MenuOption::Exit, LastOption));
        },
        BoardFinished::Loss => {
            **finished_count = 0;
            commands.spawn(MenuTitle::GameOver);
            commands.spawn_batch([
                (MenuOption::Play),
                (MenuOption::Exit)
            ]);
        },
    }
    if let Ok(score) = scores.get(player.0) {
        commands.spawn(MenuTitle::Custom(format!("Score: {}", score.0).to_string()));
    }
    game_state.set(GameState::NotStarted);
    app_state.set(AppState::Menu);
}