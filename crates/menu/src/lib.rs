use bevy::prelude::*;
use pills_core::*;
use pills_input::*;
use pills_score::*;
use pills_auras::*;

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
                        handle_game_result
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
    boards: Query<(Entity, &BoardConfig)>,
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
                    GameState::Finished | GameState::NotStarted => game_state.set(GameState::Starting),
                    GameState::Paused => game_state.set(GameState::Active),
                    _ => {},
                }
                app_state.set(AppState::InGame);
            },
            (Interaction::Pressed, MenuOption::NextLevel) => {
                let mut text = text_query.get_mut(children[0]).unwrap();
                *background_color = Color::DARK_GRAY.into();
                text.sections[0].style.color = Color::PINK.into();
                for (board, config) in boards.iter() {
                    commands
                        .spawn((
                            InBoard(board), 
                            ScorePolicy::default(),
                            LimitedMovePolicy::new(99, AuraEffect::BoardFinished(BoardFinished::Loss)),
                        ))
                    ;
                }
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

fn handle_game_result(
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut events: EventReader<BoardResult>,
    boards: Query<(Entity, Option<&KeyControlled>, Option<&GlobalScore>)>, 
) {
    let mut show_menu = false;
    for event in events.iter() {
        if let Ok((entity, maybe_key_controlled, maybe_score)) = boards.get(event.0) {
            match (maybe_key_controlled, maybe_score, event.1) {
                (Some(_), Some(score), result) => {
                    show_menu = true;
                    if result {
                        commands.spawn(MenuTitle::Victory);
                        commands.spawn_batch([
                            (MenuOption::NextLevel),
                            (MenuOption::Exit)
                        ]);
                    } else {
                        commands.spawn(MenuTitle::GameOver);
                        commands.spawn_batch([
                            (MenuOption::Play),
                            (MenuOption::Exit)
                        ]);
                        // TODO: This does not belong here
                        commands.entity(entity).insert(BoardConfig::default());
                    }
                    commands.spawn(MenuTitle::Custom(format!("Score: {}", score.0).to_string()));
                },
                _ => {}
            }
        }
    }
    if show_menu {
        game_state.set(GameState::NotStarted);
        app_state.set(AppState::Menu);
    }
}