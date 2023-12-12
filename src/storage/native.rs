//! Native implementation of the store trait

use super::{Deserialize, Serialize, Store};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// key/value store that can serialize and deseralize to a file?
// example:
// key = "value"
// key_2 = 3
pub struct FileStore {
    path: PathBuf,
    map: HashMap<String, String>,
}

impl FileStore {
    pub fn new(path: std::path::PathBuf) -> Self {
        // create config file if it doesn't exist
        if !path.exists() {
            // make sure path directories exists
            fs::create_dir_all(&path.parent().unwrap()).expect("failed to create settings dir");
            fs::File::create(&path).expect("failed to crate and open settings file");
        }

        // read file into hashmap
        let string = fs::read_to_string(&path).expect("failed to read file");
        let mut map = HashMap::new();
        for line in string.lines() {
            let mut split = line.split('=');
            let key = split.next().expect("failed to get key").trim();
            let value = split.next().expect("failed to get value").trim();
            map.insert(key.to_string(), value.to_string());
        }

        Self { path, map }
    }

    // write the map to the file
    fn write(&mut self) {
        let mut string = String::new();
        // serialize hashmap into a toml-style string
        for (key, value) in self.map.iter() {
            string.push_str(&format!("{} = {}\n", key, value));
        }

        // write string to file
        fs::write(&self.path, string).expect("failed to write to file");
    }
}

impl Store for FileStore {
    #[cfg(not(target_arch = "wasm32"))]
    fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), ()> {
        let string = value.serialize();
        self.map.insert(key.to_string(), string);
        self.write();
        Ok(())
    }

    fn get<T: Deserialize>(&self, key: &str) -> Result<T, ()> {
        let entry = self.map.get(key).ok_or(())?;
        let value = T::deserialize(entry.to_string());
        Ok(value)
    }
}
