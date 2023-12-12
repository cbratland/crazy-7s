//! WebAssembly specific implementation of the Store trait.

use super::{Deserialize, Serialize, Store};

pub struct LocalStorage;

impl LocalStorage {
    // get refrerence to web LocalStorage object
    fn storage() -> web_sys::Storage {
        web_sys::window()
            .expect("No window")
            .local_storage()
            .expect("Failed to get local storage")
            .expect("No local storage")
    }
}

impl Store for LocalStorage {
    fn get<T: Deserialize>(&self, key: &str) -> Result<T, ()> {
        let storage = Self::storage();
        let entry = storage.get_item(&key).map_err(|_| ())?;
        let string = entry.as_ref().ok_or(())?;
        let value = T::deserialize(string.to_string());
        Ok(value)
    }

    fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), ()> {
        let string = value.serialize();
        let storage = Self::storage();
        storage.set_item(&key, &string).map_err(|_| ())?;
        Ok(())
    }
}
