use bevy::prelude::*;
use bevy::sprite::Anchor;
use pills_level::*;
use pills_core::*;
use pills_ui::{ContentContainer, SidebarContainer};
use super::*;

#[derive(Default, PartialEq, Resource)]
struct BoardCount(usize);

#[derive(Component)]
struct BoardBackground(usize);

pub(crate) struct BoardSpritesPlugin;

impl Plugin for BoardSpritesPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<BoardCount>()
            .add_systems(OnEnter(GameState::Starting), add_board_sprites)
            .add_systems(
                PostUpdate, 
                update_transforms
                    .before(bevy::transform::TransformSystem::TransformPropagate))
        ;
    }
}

fn add_board_sprites(
    mut commands: Commands,
    mut board_count: ResMut<BoardCount>,
    mut level: ResMut<Level>,
    boards: Query<&BoardConfig>,
) {
    let mut num_boards = 0;
    let configs = level.board_configs.clone();
    for board_entity in configs.iter() {
        if let Ok(config) = boards.get(*board_entity) {
            let (rows, cols) = config.board_size;
            let (mut width, mut height) = (CELL_SIZE * cols as f32, CELL_SIZE * rows as f32);
            // Add space to height for the next pill
            height += CELL_SIZE * 2.0;
            // Create a border effect
            width += 8.0;
            // Spawn the background for all the board components
            let background_entity = commands
                .spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::DARK_GRAY,
                            custom_size: Some(Vec2::new(width, height)),
                            ..default()
                        },
                        transform: Transform::from_xyz(0.0, 0.0, 0.0),
                        ..default()
                    },
                    BoardBackground(num_boards),
                ))
                .id()
            ;
            let info_container = commands
                .spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::BLACK,
                            custom_size: Some(Vec2::new(width - 8.0, CELL_SIZE * 2.0 - 8.0)),
                            anchor: Anchor::TopLeft,
                            ..default()
                        },
                        transform: Transform::from_xyz(-width/2.0 + 4.0, height/2.0 - 4.0, 1.0),
                        ..default()
                    },
                ))
                .set_parent(background_entity)
                .id()
            ;
            commands.entity(*board_entity)
                .insert((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(0.4, 0.4, 0.4),
                            custom_size: Some(Vec2::new(CELL_SIZE * cols as f32, CELL_SIZE * rows as f32)),
                            ..default()
                        },
                        transform: Transform::from_xyz(0.0, -CELL_SIZE+4.0, 1.0),
                        ..default()
                    },
                    BoardInfoContainer(info_container),
                ))
                .set_parent(background_entity)
            ;
            level.root = Some(background_entity);
            num_boards += 1;
        }
    }
    board_count.set_if_neq(BoardCount(num_boards));
}

fn update_transforms(
    mut boards: Query<(&BoardBackground, &Sprite, &mut Transform), Added<BoardBackground>>,
    transforms: Query<(&GlobalTransform, &Node)>,
    content_container: Res<ContentContainer>,

) {
    if boards.is_empty() {
        return;
    }
    if let Ok((global_content_transform, content_size)) = transforms.get(content_container.0) {
        info!("Global content transform: {:?} {:?}", global_content_transform, content_size);
        for (background, sprite, mut transform) in boards.iter_mut() {
            //let mut x = -content_size.size().x/2.0;
            let mut x = 0.0;
            if let Some(size) = sprite.custom_size {
                x += (background.0 as f32) * size.x;
                info!("Board offset {:?}", x);
            }
            transform.translation = Vec3::new(x, 0.0, 1.0);
        }
    }
}