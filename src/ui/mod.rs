use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2},
    EguiContext, EguiPlugin,
};

use crate::GameState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiState {
    MainMenu,
    InGame,
    Settings,
    Paused,
    EndGame,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin);
        app.add_state(UiState::MainMenu);

        // app.add_system(ui_main_menu);

        app.insert_resource(GameSettings::default());
        app.add_system_set(SystemSet::on_update(GameState::MainMenu).with_system(ui_main_menu));
        // app.add_system_set(
        //     SystemSet::on_update(GameState::InGame)
        //         .with_system(selecting_sub_cube)
        //         .with_system(rotate_side.after(selecting_sub_cube))
        //         .with_system(stop_rotation.after(rotate_side)),
        // );
        // app.add_system_set(SystemSet::on_exit(GameState::InGame).with_system(init_rubiks_cube));
    }
}

#[derive(Debug, Default, PartialEq)]
enum ScreenMode {
    Fullscreen,
    #[default]
    Windowed,
}

#[derive(Debug, Default, Resource)]
struct GameSettings {
    mode: ScreenMode,
}

fn ui_main_menu(
    mut game_state: ResMut<State<GameState>>,
    mut ui_state: ResMut<State<UiState>>,
    mut game_settings: ResMut<GameSettings>,
    mut egui_context: ResMut<EguiContext>,
) {
    egui::Window::new("Rubik's Cube")
        .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
        .title_bar(false)
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.set_width(200.0);
            ui.set_height(200.0);

            match ui_state.current() {
                UiState::MainMenu => {
                    let _play = ui.button("Play");
                    let settings = ui.button("Settings");
                    let _exit = ui.button("Exit");

                    if settings.clicked() {
                        ui_state.push(UiState::Settings).unwrap();
                    }
                }
                UiState::Settings => {
                    let mode = &mut game_settings.mode;
                    egui::ComboBox::from_label("WindowMode")
                        .selected_text(format!("{mode:?}"))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(mode, ScreenMode::Windowed, "Windowed");
                            ui.selectable_value(mode, ScreenMode::Fullscreen, "Fullscreen");
                        });
                    let _volume = ui.button("Volume");
                    let back = ui.button("Back");
                    if back.clicked() {
                        game_state.pop().unwrap();
                    }
                }
                _ => {}
            }
        });
}
