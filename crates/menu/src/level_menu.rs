use bevy::prelude::*;
use super::*;

#[derive(Default, Deref, DerefMut, Resource)]
struct FinishedCount(u32);

pub(crate) struct LevelMenuPlugin;

impl Plugin for LevelMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<FinishedCount>()
            .add_systems(OnEnter(GameState::Finished), (setup, setup_resources))
            .add_systems(Update, (handle_interactions, add_icons).run_if(in_state(AppState::LevelMenu)))
            .add_systems(OnEnter(AppState::LevelMenu), spawn)
            .add_systems(OnExit(AppState::LevelMenu), despawn)
        ;
    }
}

#[derive(Component)]
pub(crate) struct SelectedLevelConfig(pub Entity);

pub(crate) fn add_level_button_bundle_with_icons(world: &mut World, id: Entity) {
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
    for (_, parent_id, config_id) in &menu_options {
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

#[derive(Component)]
struct LastOption;

fn setup(
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
            let frequency = Frequency { amount: 10 };
            let frequency_id = commands.spawn_empty().add(Augment::Frequency(frequency)).id();
            let level_config = LevelConfig::with_augments(vec![explosive_id, frequency_id]);
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
    app_state.set(AppState::LevelMenu);
}
