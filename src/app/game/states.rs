use bevy::prelude::*;
use bevy::state::state::{States, State, NextState, OnEnter, OnExit, OnTransition};
use bevy::state::condition::in_state;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    Loading,
    Playing,
    Paused,
    GameOver,
    Settings,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum PlayingState {
    #[default]
    Countdown,
    Active,
    Mutation,
    ShiftTransition,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum UIState {
    #[default]
    Hidden,
    Survival,
    Mutation,
    GameOver,
    Pause,
}
