use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Site {
    pub enable_registration: bool,
}

impl Default for Site {
    fn default() -> Self {
        Self {
            enable_registration: true,
        }
    }
}
