use bevy::prelude::Component;

#[derive(Component)]
pub struct LevelHistoryOption {
    pub index: usize,
}

#[derive(Component)]
pub struct PaginationOption {
    pub index: usize,
}

/// Returns to the run in progress.
#[derive(Component)]
pub struct ContinueButton;

/// Ends the run and goes to the summary.
#[derive(Component)]
pub struct EndRunButton;

#[derive(Component)]
pub struct GameHistoryMenu;

#[derive(Component)]
pub struct PaginationContainer;

/// Steps the volume down, wrapping round to full from off.
#[derive(Component)]
pub struct SoundToggleButton;

/// The label inside that button.
///
/// It exists so the label can be **written in place** rather than rebuilt. The
/// screen used to redraw itself on every press, and on the web the redraw does
/// not show: a `Text` node that is despawned and respawned in the same frame
/// keeps rendering its old glyphs, so the number never changed on screen even
/// though the volume did. Native does not have the problem, which is why it
/// went unnoticed.
#[derive(Component)]
pub struct SoundToggleLabel;
