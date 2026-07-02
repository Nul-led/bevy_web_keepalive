use bevy_app::{App, First, Plugin, Startup};
use bevy_ecs::{
    change_detection::DetectChangesMut, entity::Entity, resource::Resource, world::World,
};
use bevy_window::Window;
use bevy_winit::{EventLoopProxyWrapper, WinitUserEvent};
use std::panic;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{console, js_sys::Array, window, Blob, PageTransitionEvent, Url, Worker};

/// The `WebKeepalivePlugin` plugin creates a web worker that keeps Bevy updating even when the tab is not visible.
/// This allows a game to keep Bevy running in the background (eg. when the user is on another browser tab).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WebKeepalivePlugin {
    /// The interval of time, in milliseconds, to wake Bevy when a tab is hidden.
    ///
    /// This interval timer can be changed after the initial value is set through the [`KeepaliveSettings`] resource.
    ///
    /// The default is 16.667, or 60 updates per second.
    pub wake_delay: f64,
}

impl Default for WebKeepalivePlugin {
    fn default() -> Self {
        Self {
            wake_delay: 1000.0 / 60.0,
        }
    }
}

impl Plugin for WebKeepalivePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KeepaliveSettings {
            wake_delay: self.wake_delay,
            worker: None,
            hidden_windows: Vec::new(),
        });

        app.add_systems(Startup, system_init_background_worker);
        app.add_systems(First, restore_windows_after_keepalive);
    }
}

/// The `KeepaliveSettings` resource can be used to control at runtime how the background worker operates.
///
/// Please note that it currently isn't possible to change from `setTimeout` to `setInterval`.
#[derive(Debug, Default, Resource)]
pub struct KeepaliveSettings {
    /// The interval of time, in milliseconds, to wake Bevy when a tab is hidden.
    ///
    /// The default is 16.667, or 60 updates per second.
    pub wake_delay: f64,

    worker: Option<Worker>,
    hidden_windows: Vec<Entity>,
}

// These are safe to implement as we are in a single threaded environment, they are only needed to satisfy bevy's trait requirements for resources
unsafe impl Send for KeepaliveSettings {}
unsafe impl Sync for KeepaliveSettings {}

impl Drop for KeepaliveSettings {
    fn drop(&mut self) {
        if let Some(worker) = &self.worker {
            worker.terminate();
        }
    }
}

/// The `system_init_background_worker` system runs at `Startup` and launches the web worker with a tick loop based on `setInterval`.
fn system_init_background_worker(world: &mut World) {
    let wake_delay = world.resource::<KeepaliveSettings>().wake_delay;
    let script = Blob::new_with_str_sequence(
        &Array::of1(&JsValue::from_str(&format!(
            "
            let interval = setInterval(() => self.postMessage(null), {});
            self.onmessage = v => {{
                const delay = parseInt(v);
                if (isNaN(delay)) return;
                clearInterval(interval);
                interval = setInterval(() => self.postMessage(null), delay);
            }};
            ",
            wake_delay
        )))
        .unchecked_into(),
    )
    .unwrap();

    let worker = Worker::new(&Url::create_object_url_with_blob(&script).unwrap()).unwrap();
    install_worker_cleanup(worker.clone());

    world.resource_mut::<KeepaliveSettings>().worker = Some(worker.clone());

    let world = world as *mut World;
    let closure = Closure::<dyn FnMut()>::new(move || {
        let is_visible = window()
            .and_then(|w| w.document())
            .is_some_and(|d| !d.hidden());

        unsafe {
            let Some(world) = world.as_mut() else {
                return;
            };

            if is_visible {
                restore_windows_after_keepalive(world);
                return;
            }

            if !hide_windows_for_keepalive(world) {
                return;
            }

            let sent = world
                .get_resource::<EventLoopProxyWrapper>()
                .is_some_and(|proxy| proxy.send_event(WinitUserEvent::WakeUp).is_ok());

            if !sent {
                restore_windows_after_keepalive(world);
            }
        }
    });

    worker.set_onmessage(Some(closure.as_ref().unchecked_ref()));

    closure.forget();
}

fn install_worker_cleanup(worker: Worker) {
    let worker_on_panic = worker.clone();
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        worker_on_panic.terminate();
        previous_hook(info);
    }));

    let Some(window) = window() else {
        console::warn_1(
            &"bevy_web_keepalive: failed to install worker cleanup without a window".into(),
        );
        return;
    };

    let closure =
        Closure::<dyn FnMut(PageTransitionEvent)>::new(move |event: PageTransitionEvent| {
            if !event.persisted() {
                worker.terminate();
            }
        });

    match window.add_event_listener_with_callback("pagehide", closure.as_ref().unchecked_ref()) {
        Ok(()) => closure.forget(),
        Err(error) => console::warn_1(
            &format!("bevy_web_keepalive: failed to install pagehide worker cleanup: {error:?}")
                .into(),
        ),
    }
}

fn hide_windows_for_keepalive(world: &mut World) -> bool {
    let Some(mut settings) = world.get_resource_mut::<KeepaliveSettings>() else {
        return false;
    };
    let mut hidden_windows = std::mem::take(&mut settings.hidden_windows);

    let mut query = world.query::<(Entity, &mut Window)>();
    for (entity, mut window) in query.iter_mut(world) {
        if window.visible {
            hidden_windows.push(entity);

            // Bevy's winit runner performs a full App::update when all windows are invisible.
            // Bypass change detection so this runner hint is not synced to the backend window.
            window.bypass_change_detection().visible = false;
        }
    }

    world.resource_mut::<KeepaliveSettings>().hidden_windows = hidden_windows;

    true
}

fn restore_windows_after_keepalive(world: &mut World) {
    let Some(mut settings) = world.get_resource_mut::<KeepaliveSettings>() else {
        return;
    };
    let hidden_windows = std::mem::take(&mut settings.hidden_windows);

    if hidden_windows.is_empty() {
        return;
    }

    let mut query = world.query::<&mut Window>();
    for entity in hidden_windows {
        if let Ok(mut window) = query.get_mut(world, entity) {
            window.bypass_change_detection().visible = true;
        }
    }
}
