use bevy::prelude::*;
use bevy_web_keepalive::{KeepaliveSettings, WebKeepalivePlugin};

#[derive(Resource, Default)]
struct FrameCounter(u64);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "bevy_web_keepalive example".into(),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(WebKeepalivePlugin { wake_delay: 100.0 })
        .insert_resource(FrameCounter::default())
        .add_systems(Startup, print_instructions)
        .add_systems(Update, count_frames)
        .run();
}

fn print_instructions(settings: Res<KeepaliveSettings>) {
    info!(
        "background worker running every {}ms while the browser tab is hidden",
        settings.wake_delay
    );
    info!("open the browser console, switch tabs, and watch frames continue to advance");
}

fn count_frames(mut frames: ResMut<FrameCounter>, time: Res<Time>) {
    frames.0 += 1;

    if frames.0 == 1 || frames.0 % 60 == 0 {
        info!(
            "frame={} elapsed={:.1}s delta={:.3}s",
            frames.0,
            time.elapsed_secs(),
            time.delta_secs()
        );
    }
}
