#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "listener")]
mod background_listener;
#[cfg(feature = "listener")]
#[cfg_attr(docsrs, doc(cfg(feature = "listener")))]
pub use background_listener::{VisibilityChangeListenerPlugin, WindowVisibility};

#[cfg(feature = "timer")]
mod background_timer;
#[cfg(feature = "timer")]
#[cfg_attr(docsrs, doc(cfg(feature = "timer")))]
pub use background_timer::{BackgroundTimer, BackgroundTimerPlugin};

mod background_worker;
pub use background_worker::{KeepaliveSettings, WebKeepalivePlugin};
