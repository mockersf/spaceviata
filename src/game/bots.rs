use bevy::prelude::*;

use super::{turns::TurnState, Universe};

#[derive(Resource)]
pub struct BotTurnStatus {
    current: usize,
}

pub fn start_bots(mut commands: Commands) {
    commands.insert_resource(BotTurnStatus { current: 1 });
}

pub fn run_bots_turn(
    mut status: ResMut<BotTurnStatus>,
    universe: Res<Universe>,
    mut state: ResMut<State<TurnState>>,
) {
    warn!("playing for player {}", status.current);
    status.current += 1;
    if status.current == universe.players.len() {
        let _ = state.set(TurnState::Enemy);
    }
}
