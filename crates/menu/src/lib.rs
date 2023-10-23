use bevy::prelude::*;
use bevy::ecs::system::EntityCommand;
use pills_core::*;
use pills_level::*;
use pills_score::*;
use pills_ui::*;
use pills_augments::*;

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
                    (menu, add_icons)
                        .run_if(in_state(AppState::Menu))
                )
            )
            .add_systems(Startup, (startup_menu, setup_resources))
            .add_systems(OnEnter(AppState::Menu),  spawn_menu)
            .add_systems(OnExit(AppState::Menu), cleanup_menu)
            .add_systems(OnEnter(GameState::Finished), handle_level_finished)
        ;
    }
}

#[derive(Default, Deref, DerefMut, Resource)]
struct FinishedCount(u32);

#[derive(Clone, Component)]
enum MenuTitle {
    GameOver,
    Victory,
    Custom(String),
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

#[derive(Clone, Component)]
enum MenuOption {
    Play,
    SpecificLevel,
    Exit,
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

#[derive(Component)]
struct SelectedLevelConfig(Entity);

fn add_level_button_bundle_with_icons(world: &mut World, id: Entity) {
    world.entity_mut(id)
        .insert(
            NodeBundle {
                style: Style { 
                    width: Val::Px(320.0),
                    margin: UiRect::all(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    flex_direction: FlexDirection::Column,
                    ..default() 
                },
                border_color: Color::GRAY.into(),
                ..default()
            }
        )
        .with_children(|parent| {
            // Top Section with clickable text
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
                MenuOption::SpecificLevel,
                SelectedLevelConfig(id),
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "Play!",
                    TextStyle {
                        font_size: 40.0,
                        color: Color::WHITE.into(),
                        ..default()
                    },
                ));
            });
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

#[derive(Resource)]
struct IconAtlasHandle(Handle<TextureAtlas>);

fn setup_resources(
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
) {
    let texture_handle = asset_server.load("textures/icons.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 1, 5, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.insert_resource(IconAtlasHandle(texture_atlas_handle));
}

fn add_icons(
    mut commands: Commands,
    icons: Res<IconAtlasHandle>,
    menu_options: Query<(Entity, &Parent, &SelectedLevelConfig), (Added<MenuOption>, Added<SelectedLevelConfig>)>,
    level_configs: Query<&LevelConfig>,
    icon_indices: Query<&AugmentIconIndex>,
) {
    for (id, parent_id, config_id) in &menu_options {
        if let Ok(level_config) = level_configs.get(config_id.0) {
            // Bottom section with icons
            commands.entity(parent_id.get()).with_children(|parent| {
                parent.spawn(
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            ..default()
                        },
                        background_color: Color::DARK_GRAY.into(),
                        ..default()
                    }
                ).with_children(|parent| {
                    // Icon for each augment
                    for augment_id in &level_config.augments {
                        if let Ok(icon_index) = &icon_indices.get(*augment_id) {
                            info!("\tAugment {:?} atlas index: {:?}", augment_id, icon_index);
                            parent.spawn(
                                AtlasImageBundle {
                                    style: Style {
                                        margin: UiRect::all(Val::Px(8.0)),
                                        width: Val::Px(32.0),
                                        height: Val::Px(32.0),
                                        ..default()
                                    },
                                    texture_atlas: icons.0.clone(),
                                    texture_atlas_image: UiTextureAtlasImage { index: icon_index.0, ..default()},
                                    ..default()
                                }
                            );
                        }
                    }
                });
            });
        }
    }
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

fn spawn_menu(
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

fn menu(
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
            // Two random configs
            for _ in 0..2 {
                let mut level_config = LevelConfig::with_budget(**finished_count);
                level_config.add_random_augments(&mut commands);
                commands.spawn((MenuOption::SpecificLevel, level_config));
            }
            // One specific config
            let explosive = Volatility { 
                area: AreaOfEffect::Radius(2), 
                filter: red_viruses, 
            };
            let explosive_id = commands.spawn_empty().add(Augment::Volatility(explosive)).id();
            let frequency = Frequency { amount: 1 };
            let frequency_id = commands.spawn_empty().add(Augment::Frequency(frequency)).id();
            let level_config = LevelConfig::with_augments(vec![explosive_id, frequency_id, frequency_id, frequency_id, frequency_id, frequency_id, frequency_id, frequency_id]);
            commands.spawn((MenuOption::SpecificLevel, level_config));

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