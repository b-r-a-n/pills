use bevy::prelude::*;
use pills_core::*;

pub(crate) struct AuraPlugin;

impl Plugin for AuraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, handle_hover)
        ;
    }
}

#[derive(Component)]
struct ScoreChange(i32);


pub enum AuraEffect {
    BoardFinished(BoardFinished),
}

#[derive(Component)]
pub struct AuraIconContainer;

#[derive(Component)]
struct AuraHover;

#[derive(Component)]
struct AuraTooltip;

#[derive(Component)]
struct AuraText(String);

#[derive(Component)]
struct AuraOwner(Entity);

#[derive(Bundle)]
pub struct AuraBundle {
    text: AuraText,
    owner: AuraOwner,
}

fn _spawn_tooltip_container(
    mut commands: Commands,
) {
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "",
                TextStyle {font_size: 18.0, color: Color::WHITE, ..default()}
            ),
            style: Style {
                display: Display::None,
                position_type: PositionType::Absolute,
                max_width: Val::Vw(20.0),
                ..default()
            },
            background_color: Color::BLACK.into(),
            z_index: ZIndex::Global(1),
            ..default()
        },
        AuraTooltip,
    ));
}

pub fn spawn_layout(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
) -> Entity {
    commands.
        spawn((NodeBundle{
            style: Style {
                display: Display::Grid,
                width: Val::Px(32.0 * 10.0),
                height: Val::Px(32.0 * 2.0),
                grid_template_columns: RepeatedGridTrack::flex(10, 1.0),
                grid_template_rows: RepeatedGridTrack::flex(2, 1.0),
                ..default()
            },
            background_color: Color::GREEN.into(),
            ..default()
        }, 
        AuraIconContainer))
        .with_children(|parent| {
            let texture_handle = asset_server.load("textures/pieces.png");
            let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64., 64.), 6, 1, None, None);
            let atlas_handle = texture_atlases.add(texture_atlas);
            for i in 0..6 {
                grid_item(parent, AtlasImageBundle{
                    style: Style {
                        ..default()
                    },
                    texture_atlas: atlas_handle.clone(),
                    texture_atlas_image: UiTextureAtlasImage {index: i, ..default()},
                    ..default()
                });
            }
        })
        .id()
}

fn grid_item(builder: &mut ChildBuilder, _: impl Bundle) {
    builder
        .spawn(NodeBundle{
            style: Style {
                display: Display::Grid,
                padding: UiRect::all(Val::Px(2.)),
                ..default()
            },
            ..default()
        })
        .with_children(|builder| {
            //builder.spawn(bundle);
            //builder.spawn((NodeBundle{style: Style::default(), background_color: Color::YELLOW.into(), ..default()}, AuraHover));
            builder
                .spawn((ButtonBundle {
                    style: Style { ..default() },
                    ..default()
                }, AuraHover));
        });
}

fn handle_hover(
    interactions: Query<(Entity, &Interaction, &GlobalTransform, &AuraText), Changed<Interaction>>,
    mut tooltip: Query<(&mut Style, &mut Text), With<AuraTooltip>>,
) {
    if let Ok((mut style, mut text)) = tooltip.get_single_mut() {
        for (_, interaction, transform, aura_text) in interactions.iter() {
            match *interaction {
                Interaction::Hovered => {
                    let (x,y,_) = transform.translation().into();
                    style.display = Display::Flex;
                    style.top = Val::Px(y);
                    style.left = Val::Px(x);
                    text.sections[0].value = aura_text.0.clone();
                    return
                }
                Interaction::None => {
                    style.display = Display::None;
                }
                _ => {}
            }
        }
    }
}