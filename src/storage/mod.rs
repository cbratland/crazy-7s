//! Simple persistent key/value storage for Bevy.
//!
//! Uses a local config file for native and LocalStorage for WASM.

use bevy::prelude::*;

mod serialize;
pub use serialize::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm;

/// Generic store trait.
///
/// This is implemented for both native and wasm.
trait Store {
    fn get<T: Deserialize>(&self, key: &str) -> Result<T, ()>;
    fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), ()>;
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource)]
pub struct Storage(native::FileStore);

#[cfg(target_arch = "wasm32")]
#[derive(Resource)]
pub struct Storage(wasm::LocalStorage);

impl Storage {
    /// Creates a new storage object.
    #[cfg(target_arch = "wasm32")]
    pub fn new() -> Self {
        Self(wasm::LocalStorage)
    }

    /// Creates a new storage object.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> Self {
        let path = directories::ProjectDirs::from("com", "cbratland", "crazy7s")
            .expect("failed to get project dir")
            .data_dir()
            .to_path_buf()
            .join("settings.config");
        Self(native::FileStore::new(path))
    }

    /// Gets a value from the store.
    pub fn get<T: Deserialize>(&self, key: &str) -> Result<T, ()> {
        self.0.get(key)
    }

    /// Sets a value in the store.
    pub fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), ()> {
        self.0.set(key, value)
    }
}
