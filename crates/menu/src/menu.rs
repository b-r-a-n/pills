use bevy::prelude::*;
use pills_core::*;
use super::*;

#[derive(Clone, Component)]
pub(crate) enum MenuTitle {
    GameOver,
    Victory,
    Custom(String),
}

#[derive(Clone, Component)]
pub(crate) enum MenuOption {
    Play,
    SpecificLevel,
    Exit,
}


pub(crate) fn spawn(
    mut commands: Commands,
    menu_options: Query<(Entity, &MenuOption)>,
    menu_titles: Query<(Entity, &MenuTitle)>,
) {
    let (mut header_id, mut options_id, mut footer_id) = (None, None, None);
    let root_entity = commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()},
            background_color: Color::BLACK.into(),
            ..default()})
        .with_children(|builder| {

            // Header section
            header_id = builder.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            })
            .id().into();

            // Options section
            options_id = builder.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
                ..default()
            })
            .id().into();

            // Footer section
            footer_id = builder.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            })
            .id().into();

        })
        .id();
    for (id, title) in &menu_titles {
        commands.entity(id)
            .add(title.clone())
            .set_parent(header_id.unwrap());
    }
    for (id, option) in &menu_options {
        match option {
            MenuOption::Play | MenuOption::Exit => {
                commands.entity(id)
                    .add(MenuOptionUI)
                    .set_parent(footer_id.unwrap());
            },
            MenuOption::SpecificLevel => {
                commands.entity(id)
                    .add(MenuOptionUI)
                    .set_parent(options_id.unwrap());
            },
        }
    }
    commands.insert_resource(MenuData { root_entity });
}

pub(crate) fn handle_interactions(
    mut commands: Commands,
    mut interaction_query: Query<(Entity, &Interaction, &MenuOption, &mut BackgroundColor, &Children), (Changed<Interaction>, With<Button>)>,
    mut text_query: Query<&mut Text>,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<GameState>>,
    player_query: Query<Entity, With<Player>>,
    level_config_query: Query<&LevelConfig>,
    curr_game_state: Res<State<GameState>>,
    selected_level_config: Query<&SelectedLevelConfig>,
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
            (Interaction::Pressed, MenuOption::SpecificLevel) => {
                // TODO the level config is not on the button entity because of the heirarchy
                if let Ok(level_config_id) = selected_level_config.get(id) {
                    if let Ok(level_config) = level_config_query.get(level_config_id.0){
                        let player_ent = player_query.single();
                        let board_ent = spawn_single_board_level_with_config(&mut commands, level_config);
                        commands.entity(board_ent).insert(BoardPlayer(player_ent));
                        game_state.set(GameState::Starting);
                        app_state.set(AppState::InGame);
                    }
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

pub(crate) fn despawn(
    mut commands: Commands,
    menu: Res<MenuData>,
) {
    commands.entity(menu.root_entity).despawn_recursive();
}

fn add_text_button_bundle(world: &mut World, id: Entity, text: &str) {
    world.entity_mut(id)
        .insert(
            ButtonBundle {
                style: Style {
                    padding: UiRect {
                        top: Val::Px(10.0),
                        bottom: Val::Px(10.0),
                        ..default()
                    },
                    ..default()
                },
                ..default()
            }
        )
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                text,
                TextStyle {
                    font_size: 40.0,
                    color: Color::WHITE.into(),
                    ..default()
                },
            ));
        });
}

struct MenuOptionUI;

impl EntityCommand for MenuOptionUI {
    fn apply(self, id: Entity, world: &mut World) {
        match world.entity(id).get::<MenuOption>() {
            Some(MenuOption::SpecificLevel) => {
                add_level_button_bundle_with_icons(world, id);
            },
            Some(MenuOption::Play) => {
                add_text_button_bundle(world, id, "Play");
            },
            Some(MenuOption::Exit) => {
                add_text_button_bundle(world, id, "Exit");
            },
            _ => {}
        }
    }
}

impl EntityCommand for MenuTitle {
    fn apply(self, id: Entity, world: &mut World) {
        world.entity_mut(id).insert(TextBundle::from_section( 
            &self,
            TextStyle {
                font_size: 80.0,
                color: Color::WHITE.into(),
                ..default()
            }))
        ;
    }
}

impl Into<String> for &MenuTitle {
    fn into(self) -> String {
        match self {
            MenuTitle::GameOver => "Game Over".to_string(),
            MenuTitle::Victory => "Victory".to_string(),
            MenuTitle::Custom(text) => text.clone(),
        }
    }
}
