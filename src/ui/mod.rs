use bevy::{app::AppExit, prelude::*};
use bevy_egui::{
    egui::{self, Align2, ComboBox, Slider},
    EguiContext, EguiPlugin,
};

use crate::{
    game_settings::{GameSettings, GameSettingsEvent},
    game_state::GameState,
    GameStates,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiStates {
    MainMenu,
    InGame,
    Settings,
    Paused,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin);
        app.add_state(UiStates::MainMenu);

        app.add_system(game_ui);

        app.add_system_set(
            SystemSet::on_update(GameStates::InGame).with_system(game_keyboard_actins),
        );
    }
}

fn game_keyboard_actins(
    keys: Res<Input<KeyCode>>,
    mut game_states: ResMut<State<GameStates>>,
    mut ui_states: ResMut<State<UiStates>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        game_states.push(GameStates::Paused).unwrap();
        ui_states.push(UiStates::Paused).unwrap();
    }
}

fn game_ui(
    game_state: Res<GameState>,
    game_settings: Res<GameSettings>,
    mut game_states: ResMut<State<GameStates>>,
    mut ui_states: ResMut<State<UiStates>>,
    mut egui_context: ResMut<EguiContext>,
    mut exit_event: EventWriter<AppExit>,
    mut settings_events: EventWriter<GameSettingsEvent>,
    mut local_settings: Local<GameSettings>,
) {
    match ui_states.current() {
        UiStates::MainMenu => show_main_menu(
            &mut game_states,
            &mut ui_states,
            &mut egui_context,
            &mut exit_event,
        ),
        UiStates::InGame => show_in_game(&game_state, &mut egui_context),
        UiStates::Settings => show_settings(
            &game_settings,
            &mut ui_states,
            &mut egui_context,
            &mut settings_events,
            &mut local_settings,
        ),
        UiStates::Paused => show_paused(&mut game_states, &mut ui_states, &mut egui_context),
    }
}

fn show_main_menu(
    game_states: &mut ResMut<State<GameStates>>,
    ui_states: &mut ResMut<State<UiStates>>,
    egui_context: &mut ResMut<EguiContext>,
    exit_event: &mut EventWriter<AppExit>,
) {
    egui::Window::new("Rubik's Cube")
        .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
        .title_bar(false)
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.set_width(200.0);
            ui.set_height(200.0);

            let play = ui.button("Play");
            let settings = ui.button("Settings");
            let exit = ui.button("Exit");

            if play.clicked() {
                game_states.push(GameStates::InGame).unwrap();
                ui_states.push(UiStates::InGame).unwrap();
            }

            if settings.clicked() {
                ui_states.push(UiStates::Settings).unwrap();
            }
            if exit.clicked() {
                exit_event.send(AppExit);
            }
        });
}

fn show_in_game(game_state: &Res<GameState>, egui_context: &mut ResMut<EguiContext>) {
    egui::Window::new("Rubik's Cube")
        .anchor(Align2::CENTER_TOP, (0.0, 20.0))
        .title_bar(false)
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.set_width(200.0);
            ui.set_height(20.0);
            ui.label(format!("Solved: {}", game_state.is_solved));
        });
}

fn show_settings(
    game_settings: &Res<GameSettings>,
    ui_states: &mut ResMut<State<UiStates>>,
    egui_context: &mut ResMut<EguiContext>,
    settings_events: &mut EventWriter<GameSettingsEvent>,
    local_settings: &mut Local<GameSettings>,
) {
    egui::Window::new("Rubik's Cube")
        .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
        .title_bar(false)
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.set_width(200.0);
            ui.set_height(200.0);

            let mode = &mut local_settings.mode;
            ComboBox::from_label("WindowMode")
                .selected_text(format!("{mode:?}"))
                .show_ui(ui, |ui| {
                    ui.selectable_value(mode, WindowMode::Windowed, "Windowed");
                    ui.selectable_value(mode, WindowMode::Fullscreen, "Fullscreen");
                });

            ui.add(Slider::new(&mut local_settings.volume, 0.0..=10.0).text("Volume"));

            let apply = ui.button("Apply");
            if apply.clicked() && **local_settings != **game_settings {
                settings_events.send(GameSettingsEvent::Apply(**local_settings));
            }

            let back = ui.button("Back");
            if back.clicked() {
                ui_states.pop().unwrap();
                **local_settings = **game_settings;
            }
        });
}

fn show_paused(
    game_states: &mut ResMut<State<GameStates>>,
    ui_states: &mut ResMut<State<UiStates>>,
    egui_context: &mut ResMut<EguiContext>,
) {
    egui::Window::new("Rubik's Cube")
        .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
        .title_bar(false)
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.set_width(200.0);
            ui.set_height(200.0);

            let resume = ui.button("Resume");
            let settings = ui.button("Settings");
            let main_menu = ui.button("Main menu");

            if resume.clicked() {
                game_states.pop().unwrap();
                ui_states.pop().unwrap();
            }

            if settings.clicked() {
                ui_states.push(UiStates::Settings).unwrap();
            }
            if main_menu.clicked() {
                game_states.replace(GameStates::MainMenu).unwrap();
                ui_states.replace(UiStates::MainMenu).unwrap();
            }
        });
}
