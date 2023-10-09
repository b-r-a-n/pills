use bevy::prelude::*;
use pills_core::*;
use pills_input::*;
use pills_score::*;
use pills_auras::*;
use pills_level::*;

pub struct MenuPlugin;


impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state::<AppState>()
            .add_systems(
                Update,
                (
                    (
                        pause_game, 
                        handle_level_finished
                        .after(update_global_score)
                    )
                        .run_if(in_state(AppState::InGame)),
                    menu
                        .run_if(in_state(AppState::Menu))
                )
            )
            .add_systems(Startup, startup_menu)
            .add_systems(OnEnter(AppState::Menu), setup_menu)
            .add_systems(OnExit(AppState::Menu), cleanup_menu)
        ;
    }
}

#[derive(Component)]
enum MenuTitle {
    GameOver,
    Victory,
    Custom(String),
}

#[derive(Component)]
enum MenuOption {
    Play,
    NextLevel,
    Exit,
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
        commands.entity(entity).insert(
            TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: match title { 
                            MenuTitle::GameOver => "Game Over".to_string(), 
                            MenuTitle::Victory => "Victory".to_string(),
                            MenuTitle::Custom(text) => text.clone(),
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
            .set_parent(root_entity);
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
        .set_parent(root_entity);
    }
    commands.insert_resource(MenuData { root_entity });

}

fn menu(
    mut commands: Commands,
    mut interaction_query: Query<(&Interaction, &MenuOption, &mut BackgroundColor, &Children), (Changed<Interaction>, With<Button>)>,
    mut text_query: Query<&mut Text>,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<GameState>>,
    curr_game_state: Res<State<GameState>>,
    focused_windows: Query<(Entity, &Window)>,
){
    for (interaction, option, mut background_color, children) in &mut interaction_query {
        match (interaction, option) {
            (Interaction::Pressed, MenuOption::Play) => {
                let mut text = text_query.get_mut(children[0]).unwrap();
                *background_color = Color::DARK_GRAY.into();
                text.sections[0].style.color = Color::PINK.into();
                match curr_game_state.get() {
                    GameState::Finished | GameState::NotStarted => {
                        spawn_single_board_level(&mut commands);
                        game_state.set(GameState::Starting);
                    },
                    GameState::Paused => game_state.set(GameState::Active),
                    _ => {},
                }
                app_state.set(AppState::InGame);
            },
            (Interaction::Pressed, MenuOption::NextLevel) => {
                let mut text = text_query.get_mut(children[0]).unwrap();
                *background_color = Color::DARK_GRAY.into();
                text.sections[0].style.color = Color::PINK.into();
                spawn_single_board_level(&mut commands);
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
    commands.entity(menu.root_entity).despawn_recursive();
}

fn handle_level_finished(
    mut commands: Commands,
    mut events: EventReader<LevelFinished>,
    mut game_state: ResMut<NextState<GameState>>,
    mut app_state: ResMut<NextState<AppState>>,
    boards: Query<(Option<&KeyControlled>, Option<&GlobalScore>)>, 
) {
    use LevelFinished::*;
    if events.is_empty() { return; }
    let (game_over, score) = match events.iter().next().unwrap() {
        Win(entity) => {
            if let Ok((maybe_key_controlled, maybe_score)) = boards.get(*entity) {
                if maybe_key_controlled.is_some() {
                    (false, maybe_score.map(|s| s.0).unwrap_or(0))
                } else {
                    (true, maybe_score.map(|s| s.0).unwrap_or(0))
                }
            } else {
                unreachable!("Win event without a board entity");
            }
        },
        Loss(entity) => {
            if let Ok((maybe_key_controlled, maybe_score)) = boards.get(*entity) {
                if maybe_key_controlled.is_some() {
                    (true, maybe_score.map(|s| s.0).unwrap_or(0))
                } else {
                    (false, maybe_score.map(|s| s.0).unwrap_or(0))
                }
            } else {
                unreachable!("Loss event without a board entity");
            }

        },
        // TODO: Fix the score here...
        Draw => { (false, 0) },
    };
    if game_over {
        commands.spawn(MenuTitle::GameOver);
        commands.spawn_batch([
            (MenuOption::Play),
            (MenuOption::Exit)
        ]);
    } else {
        commands.spawn(MenuTitle::Victory);
        commands.spawn_batch([
            (MenuOption::NextLevel),
            (MenuOption::Exit)
        ]);
    }
    commands.spawn(MenuTitle::Custom(format!("Score: {}", score).to_string()));
    events.clear();
    game_state.set(GameState::NotStarted);
    app_state.set(AppState::Menu);
}