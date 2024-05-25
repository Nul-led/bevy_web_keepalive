# bevy_web_keepalive

[![crates.io](https://img.shields.io/crates/v/bevy_web_keepalive)](https://crates.io/crates/bevy_web_keepalive)
[![docs.rs](https://docs.rs/bevy_web_keepalive/badge.svg)](https://docs.rs/bevy_web_keepalive)

Library of bevy plugins to keep a bevy app running in the browser despite despite not being visible

## WebKeepalivePlugin

The `WebKeepalivePlugin` plugin creates a web worker that runs the main schedule to keep bevy running in the background (eg. when the user is on another browser tab).

Usage:

```rust
// To add the worker, use add_plugins
app.add_plugins(WebKeepalivePlugin::default())

// Configure the worker like this:
app.add_plugins(WebKeepalivePlugin {
    initial_wake_delay: 1000.0, // 1 sec delay
    use_set_timeout: false, // use setInterval internally instead of setTimeout
})

// To change the wake_delay at run-time, access the `KeepaliveSettings` resource in a system
fn system_a(worker: Res<KeepaliveSettings>) {
    worker.wake_delay = 16.667; // 60Hz updates
}
```

Reasoning: `bevy_winit` runs it's event loop via requestAnimationFrame. This works well for apps that don't need to run if they are in the background. However there are situations where this is unwanted such as multiplayer games that require a constant connection and cannot rely on reconnecting.

## VisibilityChangeListenerPlugin

The `VisibilityChangeListenerPlugin` plugin registers a listener that fires whenever the app's visibility has changed and updates the `WindowVisibility` resource while also allowing the `Main` schedule to run a last time after the app is hidden.

Usage:

```rust
// To add the listener, use add_plugins
app.add_plugins(VisibilityChangeListenerPlugin::default())

// To actually run the main schedule, configure the plugin like this:
app.add_plugins(VisibilityChangeListenerPlugin { run_main_schedule_on_hide: true })

// To use the `WindowVisibility` resource, access it in a system
fn system_a(window_visibility: Res<WindowVisibility>) {
    if !window_visibility.0 {
        // Do something that you want to do whenever the window is hidden
    }
}
```

Reasoning: This may be used to notify internal or external services of user inactivity.

Feature Requirements: `listener`

## BackgroundTimerPlugin

The `BackgroundTimerPlugin` plugin adds a utility resource which contains a timer that keeps track of time spent in the background. This plugin needs to be paired with the `WebKeepalivePlugin` to function properly (frame delta time is capped at 250ms in bevy_time by default)

Usage:

```rust

// To add the listener, use add_plugins, please note that the WebKeepalivePlugin.initial_wake_delay should be < 250.0 so that we can ensure that the frame delta time won't be capped at 250ms
app.add_plugins((WebKeepalivePlugin::default(), BackgroundTimerPlugin))

// To use the `BackgroundTimer` resource, access it in a system
fn system_a(timer: Res<BackgroundTimer>) {
    if !timer.0.elapsed_secs() > 60.0 {
        // Clientside-timeout the user or something similar
    }
}
```

Reasoning: Can be used to collect analytics about user behavior regarding window visibility or (clientside) timeout a player in multiplayer games after a certain time of inactivity.

Feature Requirements: `timer`
