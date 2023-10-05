use bevy::prelude::*;
use pills_input::KeyControlled;

use pills_game_board::*;
use pills_auras::*;
use pills_pieces::*;
use pills_core::*;
use pills_input::*;
use pills_sound::*;
use pills_menu::*;

#[derive(Component, Deref, DerefMut)]
struct CellComponent(Cell<Entity>);


/// Put systems here
/// 
fn setup_camera(
    mut commands: Commands
) {
    commands.spawn(Camera2dBundle::default());
}

const RED_COLOR : Color = Color::rgb(255.0/255.0, 115.0/255.0, 106.0/255.0);
const YELLOW_COLOR : Color = Color::rgb(255.0/255.0, 213.0/255.0, 96.0/255.0);
const BLUE_COLOR : Color = Color::rgb(0.0/255.0, 194.0/255.0, 215.0/255.0);

#[derive(Resource, Deref, DerefMut)]
struct PieceAtlasHandle(Handle<TextureAtlas>);

fn setup_resources(
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
) {
    let texture_handle = asset_server.load("textures/pieces.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 1, 6, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.insert_resource(PieceAtlasHandle(texture_atlas_handle));
}

const CELL_SIZE: f32 = 32.0;


fn add_sprites(
    mut commands: Commands,
    atlas_handle: Res<PieceAtlasHandle>,
    mut virus_query: Query<(Entity, &Virus), (Added<Virus>, With<BoardPosition>)>,
    mut pill_query: Query<(Entity, &Pill, Option<&BoardPosition>, Option<&NextPill>), Added<Pill>>,
    cleared_query: Query<(Entity, &ClearedCell), (Added<ClearedCell>, With<BoardPosition>)>,
) {
    for (entity, virus_type) in virus_query.iter_mut() {
        let (texture_atlas, sprite) = match virus_type.0 {
            CellColor::RED => (atlas_handle.clone(), TextureAtlasSprite::new(1)),
            CellColor::BLUE => (atlas_handle.clone(), TextureAtlasSprite::new(0)),
            CellColor::YELLOW => (atlas_handle.clone(), TextureAtlasSprite::new(2)),
            CellColor::GREEN => (atlas_handle.clone(), TextureAtlasSprite::new(2)),
            CellColor::ORANGE => (atlas_handle.clone(), TextureAtlasSprite::new(2)),
            CellColor::PURPLE => (atlas_handle.clone(), TextureAtlasSprite::new(1)),
        };
        let transform = Transform::from_scale(Vec3::new(0.5, 0.5, 1.0));
        commands.entity(entity)
            .insert(SpriteSheetBundle { 
                texture_atlas,
                sprite, 
                transform,
                ..default() 
        });
    }
    for (entity, pill_type, board_position, next_pill) in pill_query.iter_mut() {
        let color = match pill_type.0 {
            CellColor::RED => RED_COLOR,
            CellColor::YELLOW => YELLOW_COLOR,
            CellColor::BLUE => BLUE_COLOR,
            CellColor::ORANGE => RED_COLOR,
            CellColor::GREEN => YELLOW_COLOR,
            CellColor::PURPLE => BLUE_COLOR,
        };
        let sprite = TextureAtlasSprite {index:5, color, ..default()};
        let transform = match (board_position, next_pill) {
            (Some(pos), _) => { 
                Transform::from_xyz(pos.column as f32 * CELL_SIZE, pos.row as f32 * CELL_SIZE, 100.0)
                    .with_scale(Vec3::new(0.5, 0.5, 1.0))
            },
            (None, Some(next_index)) => {
                Transform::from_xyz((10.0 + next_index.0 as f32) * CELL_SIZE, 0.0, 100.0)
                    .with_scale(Vec3::new(0.5, 0.5, 1.0))
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2 * ((next_index.0 as f32 * 2.0) + 1.)))
            },
            _ => continue,
        };
        commands.entity(entity)
            .insert(SpriteSheetBundle { 
                texture_atlas: atlas_handle.clone(),
                sprite,
                transform,
                ..default() 
        });
    }
    for (entity, cell) in &cleared_query {
        let color = match cell.color {
            CellColor::RED => RED_COLOR,
            CellColor::YELLOW => YELLOW_COLOR,
            CellColor::BLUE => BLUE_COLOR,
            CellColor::ORANGE => RED_COLOR,
            CellColor::GREEN => YELLOW_COLOR,
            CellColor::PURPLE => BLUE_COLOR,
        };
        commands.entity(entity)
            .insert(SpriteSheetBundle {
                sprite: TextureAtlasSprite { index: 3, color, ..default()},
                texture_atlas: atlas_handle.clone(),
                transform: Transform::from_scale(Vec3::new(0.5, 0.5, 1.0)),
                ..default()
            });
    }
}

fn update_transforms(
    mut query: Query<(Entity, &BoardPosition, &mut Transform, &InBoard), Or<(Added<Transform>, Added<BoardPosition>, Changed<BoardPosition>)>>,
    boards: Query<&GameBoard>,
) {
    for (entity, board_position, mut transform, board) in query.iter_mut() {
        let board = boards.get(**board).unwrap();
        transform.translation.x = (board_position.column as f32 * CELL_SIZE) - (CELL_SIZE * board.cols as f32) / 2.0 + CELL_SIZE / 2.0;
        transform.translation.y = (board_position.row as f32 * CELL_SIZE) - (CELL_SIZE * board.rows as f32) / 2.0 + CELL_SIZE / 2.0;
        if let Some(orientation) = board
            .get(board_position.row as usize, board_position.column as usize)
            .get_orientation() {
            transform.rotation = match orientation {
                Orientation::Above => Quat::from_rotation_z(std::f32::consts::PI),
                Orientation::Below => Quat::from_rotation_z(0.),
                Orientation::Left => Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
                Orientation::Right => Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
            };
        }
    }
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
        .add_plugins(InputPlugin)
        .add_plugins(SoundPlugin)
        .add_plugins(MenuPlugin)
        .add_systems(
            Startup, 
            (
                setup_resources, 
                setup_camera, 
                setup_ui_grid.after(setup_resources)
            )
        )
        .add_systems(PostStartup, (startup_finished, spawn_game_boards))
        .add_systems(
            Update, 
            (
                add_sprites, 
                bevy::window::close_on_esc))
        .add_systems(PostUpdate, update_transforms.before(bevy::transform::TransformSystem::TransformPropagate))
        .run();
}