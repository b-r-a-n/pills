use bevy::prelude::*;

use pills_pieces::*;
use pills_core::*;
use pills_input::*;
use pills_sound::*;
use pills_menu::*;
use pills_level::*;
use pills_sprites::*;

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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LevelPlugin)
        .add_plugins(PillsSpritesPluginGroup)
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
        .add_systems(
            Update, 
            bevy::window::close_on_esc)
        .run();
}