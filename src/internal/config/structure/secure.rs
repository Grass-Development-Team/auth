use serde::{Deserialize, Serialize};

use crate::internal::utils;

fn default_jwt_secret() -> String {
    utils::rand::string(16)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Secure {
    /// JWT Secret key.
    /// If not set, a random key will be generated and stored in the config file.
    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,
}

impl Default for Secure {
    fn default() -> Self {
        Secure {
            jwt_secret: default_jwt_secret(),
        }
    }
}
