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


fn startup_finished(
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
    sidebar_container: Res<SidebarContainer>
) {
    // TODO: Figure out to make this work in the aura plugin
    // This is problematic, if we don't have the plugin group, we don't want to do this
    // It should probably be moved into the plugin itself
    let layout_ent = spawn_layout(&mut commands, &asset_server, &mut texture_atlases);
    commands.entity(sidebar_container.0).add_child(layout_ent);
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
    commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(CELL_SIZE * cols as f32, CELL_SIZE * rows as f32)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                ..default()
            },
            config,
            KeyControlled,
            ScorePolicy::default(),
        ));
}

/// Put components here
/// 




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
        .add_systems(PostStartup, (startup_finished, spawn_game_boards))
        .add_systems(
            Update, 
            bevy::window::close_on_esc)
        .run();
}