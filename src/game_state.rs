use bevy::prelude::*;

use crate::{rubiks_cube::RubiksCube, GameStates};

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameState::default());

        app.add_system_set(SystemSet::on_update(GameStates::InGame).with_system(update_game_state));
    }
}

#[derive(Debug, Default, Resource)]
pub struct GameState {
    pub is_solved: bool,
}

fn update_game_state(rubiks_cube: Query<&RubiksCube>, mut game_state: ResMut<GameState>) {
    if let Ok(rb) = rubiks_cube.get_single() {
        game_state.is_solved = rb.is_solved();
    }
}
