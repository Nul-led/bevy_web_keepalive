# bevy_web_keepalive

[![crates.io](https://img.shields.io/crates/v/bevy_web_keepalive)](https://crates.io/crates/bevy_web_keepalive)
[![docs.rs](https://docs.rs/bevy_web_keepalive/badge.svg)](https://docs.rs/bevy_web_keepalive)

Library of bevy plugins to keep a bevy app running in the browser despite despite not being visible

### Background Worker Plugin
The `BackgroundWorkerPlugin` plugin creates a web worker that runs the main schedule to keep bevy running in the background (eg. when the user is on another browser tab).

Reasoning: `bevy_winit` runs it's event loop via requestAnimationFrame. This works well for apps that don't need to run if they are in the background. However there are situations where this is unwanted such as multiplayer games that require a constant connection and cannot rely on reconnecting.

Feature Requirements: `default-features` | `worker` 

### Background Listener Plugin
The `VisiblityChangeListenerPlugin` plugin registers a listener that fires whenever the app's visibility has changed and updates the `WindowVisibility` resource while also allowing the `Main` schedule to run a last time after the app is hidden.

Reasoning: This may be used to notify internal or external services of user inactivity.

Feature Requirements: `listener`

### Background Timer Plugin
The `BackgroundTimerPlugin` plugin adds a utility resource which contains a timer that keeps track of time spent in the background. This plugin needs to be paired with the `BackgroundWorkerPlugin` to function properly (frame delta time is capped at 250ms in bevy_time by default)

Reasoning: Can be used to collect analytics about user behavior regarding window visibility or (clientside) timeout a player in multiplayer games after a certain time of inactivity.

Feature Requirements: `timer`
