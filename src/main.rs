use bevy::prelude::*;

use pills_core::*;
use pills_input::*;
use pills_sound::*;
use pills_menu::*;
use pills_level::*;
use pills_sprites::*;
use pills_ui::*;
use pills_score::*;
use pills_augments::*;

fn setup_camera(
    mut commands: Commands
) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_player(
    mut commands: Commands,
) {
    commands.spawn(Player);
}


fn print_state_change(
    state: Res<State<GameState>>,
) {
    info!("State changed to {:?}", state.get());
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PillsUiPlugin)
        .add_plugins(LevelPlugin)
        .add_plugins(PillsSpritesPluginGroup)
        .add_plugins(AugmentPlugin)
        .add_plugins(ScorePlugin)
        .add_plugins(GamePlugin)
        .add_plugins(InputPlugin)
        .add_plugins(SoundPlugin)
        .add_plugins(MenuPluginGroup)
        .add_systems(
            Startup, 
            (
                setup_camera, 
                setup_player,
                //|mut commands: Commands| { add_augment(&mut commands, SUPERBUG); },
            )
        )
        .add_systems(
            Update, (
                print_state_change.run_if(state_changed::<GameState>()),
                bevy::window::close_on_esc,
            ) 
        )
        .run();
}