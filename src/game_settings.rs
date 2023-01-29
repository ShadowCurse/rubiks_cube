use bevy::prelude::*;

use crate::ui::UiStates;

pub struct GameSettingsPlugin;

impl Plugin for GameSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameSettings::default());
        app.add_event::<GameSettingsEvent>();

        app.add_system_set(
            SystemSet::on_update(UiStates::Settings).with_system(update_game_settings),
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Resource)]
pub struct GameSettings {
    pub mode: WindowMode,
    pub volume: f64,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            mode: WindowMode::Windowed,
            volume: 2.0,
        }
    }
}

pub enum GameSettingsEvent {
    Apply(GameSettings),
}

fn update_game_settings(
    mut windows: ResMut<Windows>,
    mut game_settings: ResMut<GameSettings>,
    mut game_settings_events: EventReader<GameSettingsEvent>,
) {
    for event in game_settings_events.iter() {
        match event {
            GameSettingsEvent::Apply(new_settings) => {
                windows
                    .get_primary_mut()
                    .unwrap()
                    .set_mode(new_settings.mode);
                *game_settings = *new_settings;
            }
        }
    }
}
