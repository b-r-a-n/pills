use bevy::prelude::*;
use bevy::ecs::system::EntityCommand;
use bevy::sprite::Anchor;
use pills_game_board::*;
use pills_core::*;
use super::*;

pub struct PieceSpritesPlugin;

impl Plugin for PieceSpritesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_resources)
            .add_systems(Update, (add_sprites, remove_stack_indicator))
            .add_systems(
                PostUpdate, 
                update_transforms
                    .before(bevy::transform::TransformSystem::TransformPropagate)
            )
        ;
    }
}

const RED_COLOR : Color = Color::rgb(255.0/255.0, 115.0/255.0, 106.0/255.0);
const YELLOW_COLOR : Color = Color::rgb(255.0/255.0, 213.0/255.0, 96.0/255.0);
const BLUE_COLOR : Color = Color::rgb(0.0/255.0, 194.0/255.0, 215.0/255.0);

#[derive(Resource, Deref, DerefMut)]
struct PieceAtlasHandle(Handle<TextureAtlas>);

enum SpritePiece {
    Virus(Virus),
    Pill(Pill),
}

impl EntityCommand for SpritePiece {
    fn apply(self, id: Entity, world: &mut World) {
        match self {
            SpritePiece::Virus(virus) => {
                if let Some(atlas_handle) = world.get_resource::<PieceAtlasHandle>() {
                    let (texture_atlas, sprite) = match virus.0 {
                        CellColor::RED => (atlas_handle.0.clone(), TextureAtlasSprite::new(1)),
                        CellColor::BLUE => (atlas_handle.0.clone(), TextureAtlasSprite::new(0)),
                        CellColor::YELLOW => (atlas_handle.0.clone(), TextureAtlasSprite::new(2)),
                        CellColor::GREEN => (atlas_handle.0.clone(), TextureAtlasSprite::new(2)),
                        CellColor::ORANGE => (atlas_handle.0.clone(), TextureAtlasSprite::new(2)),
                        CellColor::PURPLE => (atlas_handle.0.clone(), TextureAtlasSprite::new(1)),
                    };
                    let transform = Transform::from_scale(Vec3::new(0.5, 0.5, 1.0));
                    world.entity_mut(id)
                        .insert(SpriteSheetBundle { 
                            texture_atlas,
                            sprite, 
                            transform,
                            ..default() 
                    });
                    if let Some(stack) = world.get::<ClearStack>(id) {
                        if stack.0 < 1 { return; }
                        world.spawn(Text2dBundle {
                            text: Text::from_section(
                                "+", 
                                TextStyle {
                                    font_size: 64.0,
                                    ..default()}),
                            transform: Transform::from_xyz(0.0, 0.0, 100.0),
                            text_anchor: Anchor::BottomLeft,
                            ..default()})
                            .set_parent(id)
                        ;
                    }
                }
            },
            SpritePiece::Pill(pill) => {
                let color = match pill.0 {
                    CellColor::RED => RED_COLOR,
                    CellColor::YELLOW => YELLOW_COLOR,
                    CellColor::BLUE => BLUE_COLOR,
                    CellColor::ORANGE => RED_COLOR,
                    CellColor::GREEN => YELLOW_COLOR,
                    CellColor::PURPLE => BLUE_COLOR,
                };
                let sprite = TextureAtlasSprite {index:5, color, ..default()};
                let transform = match (world.get::<BoardPosition>(id), world.get::<NextPill>(id)) {
                    (Some(pos), _) => { 
                        Transform::from_xyz(pos.column as f32 * CELL_SIZE, pos.row as f32 * CELL_SIZE, 100.0)
                            .with_scale(Vec3::new(0.5, 0.5, 1.0))
                    },
                    (None, Some(next_index)) => {
                        Transform::from_xyz(0.0, 0.0, 100.0)
                            .with_scale(Vec3::new(0.5, 0.5, 1.0))
                            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2 * ((next_index.0 as f32 * 2.0) + 1.)))
                    },
                    _ => { return }
                };
                if let Some(atlas_handle) = world.get_resource::<PieceAtlasHandle>() {
                    let texture_atlas = atlas_handle.0.clone();
                    world.entity_mut(id)
                        .insert(SpriteSheetBundle { 
                            texture_atlas,
                            sprite,
                            transform,
                            ..default() 
                    });
                    if let Some(stack) = world.get::<ClearStack>(id) {
                        if stack.0 < 1 { return; }
                        world.spawn(Text2dBundle {
                            text: Text::from_section(
                                "+", 
                                TextStyle {
                                    font_size: 16.0,
                                    ..default()}),
                            ..default()})
                        .set_parent(id)
                        ;
                    }
                }
            },
        }
    }
}

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

fn remove_stack_indicator(
    mut commands: Commands,
    mut removed: RemovedComponents<ClearStack>,
) {
    for entity in &mut removed {
        // TODO: Just remove the node associated with the clear stack
        commands.entity(entity).despawn_descendants();
    }
}


fn add_sprites(
    mut commands: Commands,
    atlas_handle: Res<PieceAtlasHandle>,
    mut viruses: Query<(Entity, &Virus), (Added<Virus>, With<BoardPosition>)>,
    mut pills: Query<(Entity, &Pill), Added<Pill>>,
    cleared_query: Query<(Entity, &ClearedCell), (Added<ClearedCell>, With<BoardPosition>)>,
) {
    for (id, virus) in viruses.iter_mut() {
        commands.entity(id).add(SpritePiece::Virus(virus.clone()));
    }
    for (id, pill) in pills.iter_mut() {
        commands.entity(id).add(SpritePiece::Pill(pill.clone()));
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