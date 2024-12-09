use bevy_app::{App, Main, Plugin, Startup};
use bevy_ecs::{event::Event, system::Resource, world::World};
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::window;

/// The `VisibilityChangeListenerPlugin` plugin registers a listener that fires when bevy's visibility is changed (eg. the user switches to a different browser tab)
///
/// The user may decide to run the `Main` schedule once after the visibility changes to hidden.
///
/// ## Panics
/// Panics if used in a headless environment (aka there is no access to window.document available)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct VisibilityChangeListenerPlugin {
    pub run_main_schedule_on_hide: bool,
}

impl Plugin for VisibilityChangeListenerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource::<WindowVisibility>(WindowVisibility(true));

        app.add_systems(
            Startup,
            match self.run_main_schedule_on_hide {
                true => system_init_active_background_listener,
                false => system_init_passive_background_listener,
            },
        );
    }
}

/// The `WindowVisibility` resource keeps track of the app's visibility
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Resource, Event)]
pub struct WindowVisibility(bool);

/// The `system_init_active_background_listener` system initializes the visibilitychange listener which runs the `Main` schedule once when hidden
fn system_init_active_background_listener(world: &mut World) {
    let world_ptr = Rc::new(world as *mut World);
    let closure = Closure::<dyn FnMut()>::new({
        let world = world_ptr.clone();
        move || {
            let window = window().expect("Unable to access the window");
            let document = window
                .document()
                .expect("Unable to access the document, is the app running in headless mode?");
            let is_hidden = document.hidden();

            unsafe {
                let Some(world) = world.as_mut() else {
                    return;
                };

                world.resource_mut::<WindowVisibility>().0 = !is_hidden;

                world.trigger(*world.resource::<WindowVisibility>());
                
                if is_hidden {
                    world.run_schedule(Main);
                }
            }
        }
    });

    let window = window().expect("Unable to access the window");
    let document = window
        .document()
        .expect("Unable to access the document, is the app running in headless mode?");

    document
        .add_event_listener_with_callback("visibilitychange", closure.as_ref().unchecked_ref())
        .expect("Unable to register event listener");

    closure.forget();
}

/// The `system_init_active_background_listener` system initializes the visibilitychange listener which doesn't run the `Main` schedule
fn system_init_passive_background_listener(world: &mut World) {
    let world_ptr = Rc::new(world as *mut World);
    let closure = Closure::<dyn FnMut()>::new({
        let world = world_ptr.clone();
        move || {
            let window = window().expect("Unable to access the window");
            let document = window
                .document()
                .expect("Unable to access the document, is the app running in headless mode?");

            unsafe {
                let Some(world) = world.as_mut() else {
                    return;
                };

                world.resource_mut::<WindowVisibility>().0 = !document.hidden();

                world.trigger(*world.resource::<WindowVisibility>());
            }
        }
    });

    let window = window().expect("Unable to access the window");
    let document = window
        .document()
        .expect("Unable to access the document, is the app running in headless mode?");

    document
        .add_event_listener_with_callback("visibilitychange", closure.as_ref().unchecked_ref())
        .expect("Unable to register event listener");

    closure.forget();
}
