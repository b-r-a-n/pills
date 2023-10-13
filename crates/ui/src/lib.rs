use bevy::prelude::*;

pub struct PillsUiPlugin;

impl Plugin for PillsUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, handle_hover)
        ;
    }
}

#[derive(Component)]
pub struct Tooltip(pub String);

#[derive(Component)]
pub struct TooltipContainer;

fn spawn_tooltip(
    commands: &mut Commands,
    parent: Entity,
    value: String,
    offset: Vec2,
) {
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                value,
                TextStyle {font_size: 18.0, color: Color::WHITE, ..default()}
            ),
            style: Style {
                display: Display::Flex,
                position_type: PositionType::Absolute,
                max_width: Val::Vw(20.0),
                left: Val::Px(offset.x/2.0),
                ..default()
            },
            background_color: Color::rgba(0., 0., 0., 0.80).into(),
            z_index: ZIndex::Global(1),
            ..default()
        },
        TooltipContainer,
    ))
    .set_parent(parent)
    ;
}

fn handle_hover(
    mut commands: Commands,
    interactions: Query<(Entity, &Interaction, &Node, &Tooltip), Changed<Interaction>>,
    containers: Query<(Entity, &Parent), With<TooltipContainer>>,
) {
    for (id, interaction, node, tooltip) in interactions.iter() {
        match *interaction {
            Interaction::Hovered => {
                let offset = node.size();
                spawn_tooltip(&mut commands, id, tooltip.0.clone(), offset);
            }
            Interaction::None => {
                for (container_id, parent_id) in &containers {
                    if parent_id.get() == id {
                        commands.entity(container_id).despawn_recursive();
                    }
                }
            }
            _ => {}
        }
    }
}

fn spawn_tooltip_container(
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
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            background_color: Color::rgba(0., 0., 0., 0.5).into(),
            z_index: ZIndex::Global(1),
            ..default()
        },
        TooltipContainer,
    ));
}