use std::sync::{Arc, RwLock};

use bevy_app::{App, Main, Plugin, Startup};
use bevy_ecs::{system::Resource, world::World};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{js_sys::Array, window, Blob, Url, Worker};

/// The `BackgroundWorkerPlugin` plugin creates a web worker that runs the main schedule every `scheduler_delay`
/// to keep bevy running in the background (eg. when the user is on another browser tab).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BackgroundWorkerPlugin {
    /// Equivalent of frame delta time (milliseconds), eg. 60Hz = 16.667
    pub initial_wake_delay: f64,
    /// Use setTimeout instead of setInterval to enable changing the scheduler delay mid-run without clearing the interval
    pub use_set_timeout: bool,
}

impl Default for BackgroundWorkerPlugin {
    fn default() -> Self {
        Self {
            initial_wake_delay: 16.667,
            use_set_timeout: true,
        }
    }
}

impl Plugin for BackgroundWorkerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BackgroundWorker {
            wake_delay: self.initial_wake_delay,
        });

        app.add_systems(
            Startup,
            match self.use_set_timeout {
                true => system_init_timeout_background_worker,
                false => system_init_interval_background_worker,
            },
        );
    }
}

/// The `BackgroundWorker` resource can be used to control at runtime how the background worker operates.
///
/// Please note that it currently isn't possible to change from `setTimeout` to `setInterval`.
#[derive(Clone, Copy, Debug, PartialEq, Default, Resource)]
pub struct BackgroundWorker {
    /// Equivalent of frame delta time (milliseconds), eg. 60Hz = 16.667
    pub wake_delay: f64,
}

/// The `system_init_timeout_background_worker` system runs at `Startup` and launches the web worker with a tick loop based on `setTimeout`
fn system_init_timeout_background_worker(world: &mut World) {
    let world_ptr = Arc::new(RwLock::new(world as *mut World));

    let worker = world.resource::<BackgroundWorker>();

    let blob = Blob::new_with_str_sequence(
        &Array::of1(&JsValue::from_str(&format!(
            "
            let delay = {};
            self.onmessage = v => {{
                const _delay = parseInt(v);
                if (!isNaN(_delay)) delay = _delay;
            }};
            const update = () => setTimeout(update, delay) && self.postMessage(null);
            setTimeout(update, delay);
            ",
            worker.wake_delay
        )))
        .unchecked_into(),
    )
    .unwrap();

    let worker = Worker::new(&Url::create_object_url_with_blob(&blob).unwrap()).unwrap();

    let closure = Closure::<dyn FnMut()>::new(move || {
        let world = unsafe { world_ptr.write().unwrap().as_mut().unwrap() };

        if window().and_then(|w| w.document()).is_some_and(|d| !d.hidden()) {
            return;
        }

        world.run_schedule(Main);
    });

    worker.set_onmessage(Some(closure.as_ref().unchecked_ref()));

    closure.forget();
}

/// The `system_init_timeout_background_worker` system runs at `Startup` and launches the web worker with a tick loop based on `setInterval`
fn system_init_interval_background_worker(world: &mut World) {
    let world_ptr = Arc::new(RwLock::new(world as *mut World));

    let worker = world.resource::<BackgroundWorker>();

    let blob = Blob::new_with_str_sequence(
        &Array::of1(&JsValue::from_str(&format!(
            "
            let interval = setInterval(self.postMessage(null), {});
            self.onmessage = v => {{
                const delay = parseInt(v);
                if (isNaN(delay)) return;
                clearInterval(interval);
                interval = setInterval(self.postMessage(null), delay);
            }};
            ",
            worker.wake_delay
        )))
        .unchecked_into(),
    )
    .unwrap();

    let worker = Worker::new(&Url::create_object_url_with_blob(&blob).unwrap()).unwrap();

    let closure = Closure::<dyn FnMut()>::new(move || {
        let world = unsafe { world_ptr.write().unwrap().as_mut().unwrap() };

        if window().and_then(|w| w.document()).is_some_and(|d| !d.hidden()) {
            return;
        }

        world.run_schedule(Main);
    });

    worker.set_onmessage(Some(closure.as_ref().unchecked_ref()));

    closure.forget();
}
