use bevy::prelude::*;
use pills_core::*;

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_resources)
            .add_systems(Update, (play_pill_sound, play_clear_sound));
    }
}

#[derive(Resource)]
struct AssetHandles {
    move_sound: Handle<AudioSource>,
    rotate_sound: Handle<AudioSource>,
    pop_sound: Handle<AudioSource>,
}

fn setup_resources(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let move_sound = asset_server.load("sounds/move.ogg");
    let rotate_sound = asset_server.load("sounds/rotate.ogg");
    let pop_sound = asset_server.load("sounds/pop.ogg");
    commands.insert_resource(AssetHandles { move_sound, rotate_sound, pop_sound });
}

fn play_pill_sound(
    mut commands: Commands,
    mut events: EventReader<PillEvent>,
    sound_handles: Res<AssetHandles>,
) {
    if events.is_empty() { return }
    for event in events.iter() {
        match event {
            PillEvent::PillMoved(_) => {
                commands.spawn(AudioBundle {
                    source: sound_handles.move_sound.clone(),
                    settings: PlaybackSettings::DESPAWN,
                });
            }
            PillEvent::PillRotated(_) =>  {
                commands.spawn(AudioBundle {
                    source: sound_handles.rotate_sound.clone(),
                    settings: PlaybackSettings::DESPAWN,
                });
            }
        }
    }
}

fn play_clear_sound(
    mut commands: Commands,
    mut events: EventReader<ClearEvent>,
    sound_handles: Res<AssetHandles>,
) {
    if events.is_empty() { return }
    commands.spawn(AudioBundle {
        source: sound_handles.pop_sound.clone(),
        settings: PlaybackSettings::DESPAWN,
    });
    events.clear();
}