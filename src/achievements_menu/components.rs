use bevy::prelude::Component;

/// Root of the goals screen. Everything under it is despawned together.
#[derive(Component)]
pub struct AchievementsMenu;

/// Returns to the main menu.
#[derive(Component)]
pub struct AchievementsBackButton;
