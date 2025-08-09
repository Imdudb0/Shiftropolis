use bevy::prelude::*;

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
