use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct ClientId(String);

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ClientId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct SharedKey(String);

impl std::fmt::Display for SharedKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "roxi-XXX")
    }
}

impl From<&str> for SharedKey {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl Into<Vec<u8>> for SharedKey {
    fn into(self) -> Vec<u8> {
        self.0.into_bytes()
    }
}
