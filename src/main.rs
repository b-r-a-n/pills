use bevy::prelude::*;
use pills_input::KeyControlled;

use pills_auras::*;
use pills_pieces::*;
use pills_core::*;
use pills_input::*;
use pills_sound::*;
use pills_menu::*;

/// Put systems here
/// 
fn setup_camera(
    mut commands: Commands
) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Resource, Deref, DerefMut)]
struct ContentContainer(Entity);

#[derive(Resource, Deref, DerefMut)]
struct SidebarContainer(Entity);

fn setup_ui_grid(
    mut commands: Commands,
) {
    let sidebar = commands
        .spawn(NodeBundle{
            style: Style {
                display: Display::Grid,
                ..default()
            },
            background_color: Color::BLUE.into(),
            ..default()
        })
        .id();
    commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                width: Val::Auto,
                height: Val::Percent(100.0),
                ..default()
            },
            background_color: Color::PURPLE.into(),
            ..default()
        })
        .add_child(sidebar);
    commands.insert_resource(SidebarContainer(sidebar));
    //commands.insert_resource(ContentContainer(content));
}

fn spawn_game_boards(
    mut commands: Commands,
){
    let config = BoardConfig::default();
    let (rows, cols) = config.board_size;
    let (mut width, mut height) = (CELL_SIZE * cols as f32, CELL_SIZE * rows as f32);
    // Add space to height for the next pill
    height += CELL_SIZE * 2.0;
    // Create a border effect
    width += 8.0;
    // Spawn the background for all the board components
    commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::PINK,
                    custom_size: Some(Vec2::new(width, height)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
        ))
        .with_children(|builder| {
            builder
                .spawn((
                    SpriteBundle {
                            sprite: Sprite {
                                color: Color::BLACK,
                                custom_size: Some(Vec2::new(CELL_SIZE * cols as f32, CELL_SIZE * rows as f32)),
                                ..default()
                            },
                        transform: Transform::from_xyz(0.0, -CELL_SIZE+4.0, 1.0),
                        ..default()
                    },
                    config,
                    KeyControlled,
                ))
            ;
        }
    );
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        //.add_plugins(AuraPluginGroup)
        //.add_plugins(ScorePlugin)
        .add_plugins(GamePlugin)
        .add_plugins(PiecesPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(SoundPlugin)
        .add_plugins(MenuPlugin)
        .add_systems(
            Startup, 
            (
                setup_camera, 
                setup_ui_grid,
            )
        )
        .add_systems(PostStartup,  spawn_game_boards)
        .add_systems(
            Update, 
            bevy::window::close_on_esc)
        .run();
}