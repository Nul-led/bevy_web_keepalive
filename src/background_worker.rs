use bevy_app::{App, Main, Plugin, Startup};
use bevy_ecs::{system::Resource, world::World};
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{js_sys::Array, window, Blob, Url, Worker};

/// The `WebKeepalivePlugin` plugin creates a web worker that runs the main schedule even when the tab is not visible.
/// This allows a game  to keep bevy running in the background (eg. when the user is on another browser tab).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WebKeepalivePlugin {
    /// The interval of time, in milliseconds, to run the `Main` schedule when a tab is hidden.
    ///
    /// This interval timer can be changed after the initial value is set through the [`KeepaliveSettings`] resource.
    ///
    /// The default is 16.667, or 60 updates per seconds.
    pub initial_wake_delay: f64,
    /// Use setTimeout instead of setInterval to enable changing the scheduler delay mid-run without clearing the interval
    pub use_set_timeout: bool,
}

impl Default for WebKeepalivePlugin {
    fn default() -> Self {
        Self {
            initial_wake_delay: 16.667,
            use_set_timeout: true,
        }
    }
}

impl Plugin for WebKeepalivePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KeepaliveSettings {
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

/// The `KeepaliveSettings` resource can be used to control at runtime how the background worker operates.
///
/// Please note that it currently isn't possible to change from `setTimeout` to `setInterval`.
#[derive(Clone, Copy, Debug, PartialEq, Default, Resource)]
pub struct KeepaliveSettings {
    /// The interval of time, in milliseconds, to run the `Main` schedule when a tab is hidden.
    ///
    /// The default is 16.667, or 60 updates per seconds.
    pub wake_delay: f64,
}

/// The `system_init_timeout_background_worker` system runs at `Startup` and launches the web worker with a tick loop based on `setTimeout`
fn system_init_timeout_background_worker(world: &mut World) {
    let setings = world.resource::<KeepaliveSettings>();
    let script = Blob::new_with_str_sequence(
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
            setings.wake_delay
        )))
        .unchecked_into(),
    )
    .unwrap();
    let worker = Worker::new(&Url::create_object_url_with_blob(&script).unwrap()).unwrap();

    let world_ptr = Rc::new(world as *mut World);
    let closure = Closure::<dyn FnMut()>::new({
        let world = world_ptr.clone();
        move || {
            if window()
                .and_then(|w| w.document())
                .is_some_and(|d| !d.hidden())
            {
                return;
            }
            unsafe {
                let Some(world) = world.as_mut() else {
                    return;
                };
                world.run_schedule(Main);
            }
        }
    });

    worker.set_onmessage(Some(closure.as_ref().unchecked_ref()));

    closure.forget();
}

/// The `system_init_timeout_background_worker` system runs at `Startup` and launches the web worker with a tick loop based on `setInterval`
fn system_init_interval_background_worker(world: &mut World) {
    let setings = world.resource::<KeepaliveSettings>();
    let script = Blob::new_with_str_sequence(
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
            setings.wake_delay
        )))
        .unchecked_into(),
    )
    .unwrap();

    let worker = Worker::new(&Url::create_object_url_with_blob(&script).unwrap()).unwrap();

    let world_ptr = Rc::new(world as *mut World);
    let closure = Closure::<dyn FnMut()>::new({
        let world = world_ptr.clone();
        move || {
            if window()
                .and_then(|w| w.document())
                .is_some_and(|d| !d.hidden())
            {
                return;
            }
            unsafe {
                let Some(world) = world.as_mut() else {
                    return;
                };
                world.run_schedule(Main);
            }
        }
    });

    worker.set_onmessage(Some(closure.as_ref().unchecked_ref()));

    closure.forget();
}
