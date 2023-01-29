use bevy::prelude::*;
use bevy_kira_audio::{prelude::AudioPlugin as KiraAudioPlugin, AudioSource};

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(KiraAudioPlugin);
        app.add_startup_system(init_game_sounds);
    }
}

#[derive(Debug, Default, Resource)]
pub struct GameSounds {
    pub rotation: Handle<AudioSource>,
}

fn init_game_sounds(asset_server: Res<AssetServer>, mut commands: Commands) {
    let rotation = asset_server.load("rotation.ogg");
    commands.insert_resource(GameSounds { rotation });
}
