use bevy_app::{App, Plugin, Update};
use bevy_ecs::system::{Res, ResMut, Resource};
use bevy_time::{Stopwatch, Time};
use web_sys::window;

/// The `BackgroundTimerPlugin` plugin creates a timer that keeps track of the time the app isn't in focus (aka in background).
///
/// To function properly running a background worker is REQUIRED.
///
/// It may prove to be useful to establish timeouts for inactive users in multiplayer games.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BackgroundTimerPlugin;

impl Plugin for BackgroundTimerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BackgroundTimer::default());

        app.add_systems(Update, system_background_timer);
    }
}

/// The `BackgroundTimer` contains a `Stopwatch` that keeps track of the time bevy ran in the background
#[derive(Clone, Debug, PartialEq, Default, Resource)]
pub struct BackgroundTimer(pub Stopwatch);

/// The `system_background_timer` system updates the `Stopwatch` based on the documents visibility
fn system_background_timer(mut timer: ResMut<BackgroundTimer>, time: Res<Time>) {
    match window()
        .and_then(|w| w.document())
        .is_some_and(|d| d.hidden())
    {
        true => _ = timer.0.tick(time.delta()),
        false => timer.0.reset(),
    };
}
