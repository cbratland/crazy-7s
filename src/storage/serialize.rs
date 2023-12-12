//! Serialize and deserialize trait

pub trait Serialize {
    /// Serialize object to a string.
    fn serialize(&self) -> String;
}

pub trait Deserialize {
    /// Deserialize object from a string.
    fn deserialize(from_string: String) -> Self;
}

// String
impl Serialize for String {
    fn serialize(&self) -> String {
        format!("\"{}\"", self)
    }
}

impl Deserialize for String {
    fn deserialize(from_string: String) -> Self {
        from_string[1..from_string.len() - 1].to_string()
    }
}

impl Serialize for &str {
    fn serialize(&self) -> String {
        format!("\"{self}\"")
    }
}

// i32
impl Serialize for i32 {
    fn serialize(&self) -> String {
        self.to_string()
    }
}

impl Deserialize for i32 {
    fn deserialize(from_string: String) -> Self {
        from_string.parse().unwrap()
    }
}

// f32
impl Serialize for f32 {
    fn serialize(&self) -> String {
        self.to_string()
    }
}

impl Deserialize for f32 {
    fn deserialize(from_string: String) -> Self {
        from_string.parse().unwrap()
    }
}

// bool
impl Serialize for bool {
    fn serialize(&self) -> String {
        self.to_string()
    }
}

impl Deserialize for bool {
    fn deserialize(from_string: String) -> Self {
        from_string.parse().unwrap()
    }
}
