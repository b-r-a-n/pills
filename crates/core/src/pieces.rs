use bevy::prelude::*;
use pills_game_board::*;

pub struct PiecesPlugin;

impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_resources)
            .add_systems(Update, add_sprites)
            .add_systems(
                PostUpdate, 
                update_transforms
                    .before(bevy::transform::TransformSystem::TransformPropagate)
            )
        ;
    }
}

#[derive(Component, PartialEq)]
pub struct BoardPosition {
    pub row: u8,
    pub column: u8,
}

#[derive(Clone, Copy, Component)]
pub struct Virus(pub CellColor);

#[derive(Clone, Copy, Component)]
pub struct Pill(pub CellColor);

#[derive(Clone, Copy, Component)]
pub struct ClearedCell {
    pub color: CellColor,
    pub was_virus: bool,
}

#[derive(Component, Deref, DerefMut)]
pub struct GameBoard(pub Board<Entity>);

#[derive(Component)]
pub struct NextPill(pub u8);

#[derive(Component, Deref, DerefMut)]
pub struct InBoard(pub Entity);

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

pub const CELL_SIZE: f32 = 32.0;

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
                Transform::from_xyz(0.0, 0.0, 100.0)
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
    mut query: Query<(&BoardPosition, &mut Transform, &InBoard), (Without<NextPill>, Or<(Added<Transform>, Added<BoardPosition>, Changed<BoardPosition>)>)>,
    mut next_pieces: Query<(&mut Transform, &NextPill, &InBoard), Or<(Added<NextPill>, Added<Transform>)>>,
    boards: Query<&GameBoard>,
) {
    for (board_position, mut transform, board) in query.iter_mut() {
        let board = boards.get(**board).unwrap();
        transform.translation.x = (board_position.column as f32 * CELL_SIZE) - (CELL_SIZE * board.cols as f32) / 2.0 + CELL_SIZE / 2.0;
        transform.translation.y = (board_position.row as f32 * CELL_SIZE) - (CELL_SIZE * board.rows as f32) / 2.0 + CELL_SIZE / 2.0;
        transform.translation.z = 100.0;
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

    for (mut transform, next_pill, board) in next_pieces.iter_mut() {
        let board = boards.get(**board).unwrap();
        let x = ((board.cols as f32 / 2.0) + next_pill.0 as f32 - 1.5) * CELL_SIZE;
        let y = (board.rows as f32 / 2.0 + 0.5) * CELL_SIZE;
        transform.translation.x = x;
        transform.translation.y = y;
        transform.translation.z = 100.0;
    }
}